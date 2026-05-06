// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Host integration (std feature).
//!
//! Provides `connect_local` for hosted OS and `InMemoryFixture` for tests.
//!
//! # Poison recovery
//!
//! All mutex operations use `Mutex::lock()` with poison recovery via
//! `into_inner()`. A poisoned mutex does **not** panic — it logs the
//! condition (via `eprintln!` in debug builds) and continues with the
//! recovered guard value. This is critical for BCI systems where a
//! panic in one task must not kill the entire process.

use crate::error::{Error, Result, TransportFault};
use crate::intent::IntentObservation;
use crate::manifest::Manifest;
use crate::stream::{IntentStream, StreamConfig, Subscription, SubscriptionId};
use std::sync::Mutex;

/// Default Unix endpoint.
pub const DEFAULT_UNIX_ENDPOINT: &str = "/var/run/axonos.sock";

/// Default Windows endpoint.
#[cfg(windows)]
pub const DEFAULT_WINDOWS_ENDPOINT: &str = r"\\.\pipe\axonos";

/// Env var override.
pub const ENDPOINT_ENV: &str = "AXONOS_ENDPOINT";

/// Connect to local AxonOS kernel.
///
/// # Errors
/// - `TransportUnreachable` if endpoint unavailable.
/// - `AbiMismatch` if ABI version incompatible.
/// - `ManifestRejected` if kernel rejects manifest.
pub fn connect_local(manifest: &Manifest, config: StreamConfig) -> Result<IntentStream> {
    let _endpoint = resolve_endpoint();

    // No TOCTOU: we do not probe the endpoint. Real IPC will be wired
    // when the kernel ships.
    if !fixture_installed() {
        return Err(Error::TransportUnreachable(TransportFault::EndpointNotFound));
    }

    let mut stream = IntentStream::new(manifest, config);
    let sub = Subscription {
        id: SubscriptionId::from_raw(next_subscription_id()),
        _not_send: core::marker::PhantomData,
    };
    stream.attach_subscription(sub);
    Ok(stream)
}

fn resolve_endpoint() -> String {
    if let Ok(v) = std::env::var(ENDPOINT_ENV) {
        return v;
    }
    #[cfg(windows)]
    {
        DEFAULT_WINDOWS_ENDPOINT.to_string()
    }
    #[cfg(not(windows))]
    {
        DEFAULT_UNIX_ENDPOINT.to_string()
    }
}

// ─── Test fixture with poison recovery ─────────────────────────────────────

static FIXTURE: Mutex<Option<InMemoryFixture>> = Mutex::new(None);
static NEXT_SUB_ID: Mutex<u64> = Mutex::new(1);

/// Recover from poisoned mutex — log and continue.
///
/// # Safety note
/// This is not `unsafe` in Rust terms, but it is a **system-level safety**
/// decision: we choose availability over strict consistency. A poisoned
/// fixture mutex means a previous test panicked while holding the lock.
/// We recover the data and continue, because killing the test runner
/// provides no value.
fn recover_fixture_lock() -> Option<InMemoryFixture> {
    match FIXTURE.lock() {
        Ok(g) => g.clone(),
        Err(poisoned) => {
            #[cfg(debug_assertions)]
            eprintln!("[axonos-sdk] WARNING: fixture mutex poisoned — recovering");
            Some(poisoned.into_inner().clone().unwrap_or(None)?)
        }
    }
}

fn fixture_installed() -> bool {
    match FIXTURE.lock() {
        Ok(g) => g.is_some(),
        Err(poisoned) => {
            #[cfg(debug_assertions)]
            eprintln!("[axonos-sdk] WARNING: fixture mutex poisoned — recovering");
            poisoned.into_inner().is_some()
        }
    }
}

fn next_subscription_id() -> u64 {
    match NEXT_SUB_ID.lock() {
        Ok(mut g) => {
            let n = *g;
            *g = g.wrapping_add(1);
            n
        }
        Err(poisoned) => {
            #[cfg(debug_assertions)]
            eprintln!("[axonos-sdk] WARNING: sub_id mutex poisoned — recovering");
            let mut g = poisoned.into_inner();
            let n = *g;
            *g = g.wrapping_add(1);
            n
        }
    }
}

/// Scripted in-memory observation source for integration tests.
#[derive(Debug, Default, Clone)]
pub struct InMemoryFixture {
    observations: Vec<IntentObservation>,
}

impl InMemoryFixture {
    #[must_use]
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    pub fn push(&mut self, obs: IntentObservation) {
        self.observations.push(obs);
    }

    pub fn install(self) {
        match FIXTURE.lock() {
            Ok(mut g) => *g = Some(self),
            Err(poisoned) => {
                #[cfg(debug_assertions)]
                eprintln!("[axonos-sdk] WARNING: fixture mutex poisoned during install");
                *poisoned.into_inner() = Some(self);
            }
        }
    }

    pub fn uninstall() {
        match FIXTURE.lock() {
            Ok(mut g) => *g = None,
            Err(poisoned) => {
                #[cfg(debug_assertions)]
                eprintln!("[axonos-sdk] WARNING: fixture mutex poisoned during uninstall");
                *poisoned.into_inner() = None;
            }
        }
    }

    #[must_use]
    pub fn pending(&self) -> usize {
        self.observations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, Direction, Manifest};
    use crate::time::MonotonicTimestamp;

    fn test_manifest() -> Manifest {
        Manifest::builder()
            .app_id("com.test.host")
            .capability(Capability::Navigation)
            .max_rate_hz(10)
            .build()
            .unwrap()
    }

    #[test]
    fn connect_without_fixture_returns_transport_error() {
        InMemoryFixture::uninstall();
        let m = test_manifest();
        let r = connect_local(&m, StreamConfig::default());
        assert!(matches!(r, Err(Error::TransportUnreachable(TransportFault::EndpointNotFound))));
    }

    #[test]
    fn connect_with_fixture_succeeds() {
        let mut fx = InMemoryFixture::new();
        let ts = MonotonicTimestamp::from_micros_unchecked(100);
        fx.push(IntentObservation::new_direction(ts, Direction::Right, 45875, 1, [0u8; 8]));
        fx.install();

        let m = test_manifest();
        let stream = connect_local(&m, StreamConfig::default()).unwrap();
        assert!(stream.is_connected());

        InMemoryFixture::uninstall();
    }

    #[test]
    fn endpoint_env_override() {
        std::env::set_var(ENDPOINT_ENV, "/tmp/test.sock");
        assert_eq!(resolve_endpoint(), "/tmp/test.sock");
        std::env::remove_var(ENDPOINT_ENV);
    }
}
