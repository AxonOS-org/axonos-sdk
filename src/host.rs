// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Host integration (std feature).
//!
//! Provides `connect_local` for hosted OS and `InMemoryFixture` for tests.
//!
//! # Mutex poisoning policy
//!
//! Earlier versions of this module silently recovered from poisoned mutexes
//! via `into_inner()`, which could mask data races in production code.
//!
//! Current policy:
//!
//! - **Test fixtures** (`InMemoryFixture`): poisoning surfaces a clear error
//!   (`Error::TransportUnreachable`).
//!   Tests should isolate their fixtures and not rely on silent recovery.
//! - **Subscription counter** (`NEXT_SUB_ID`): poisoning returns an error.
//!   A new subscription cannot be issued while the counter is in an
//!   indeterminate state.
//!
//! Production-grade IPC will use `parking_lot::Mutex` (poison-free), or
//! a lock-free counter, when the kernel ABI ships.

use crate::error::{Error, Result, TransportFault};
use crate::intent::IntentObservation;
use crate::manifest::Manifest;
use crate::stream::{IntentStream, StreamConfig, Subscription, SubscriptionId};
use std::borrow::Cow;
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
/// - [`TransportFault::EndpointNotFound`] if endpoint unavailable.
/// - [`TransportFault::Internal`] if internal state is corrupted (poisoned mutex).
pub fn connect_local(manifest: &Manifest, config: StreamConfig) -> Result<IntentStream> {
    let _endpoint = resolve_endpoint();

    // No TOCTOU: we do not probe the endpoint. Real IPC will be wired
    // when the kernel ships.
    if !fixture_installed()? {
        return Err(Error::TransportUnreachable(
            TransportFault::EndpointNotFound,
        ));
    }

    let mut stream = IntentStream::new(manifest, config);
    let sub = Subscription {
        id: SubscriptionId::from_raw(next_subscription_id()?),
        _not_send: core::marker::PhantomData,
    };
    stream.attach_subscription(sub);
    Ok(stream)
}

/// Resolve the configured endpoint.
///
/// Returns a `Cow<'static, str>` to avoid an allocation when the default
/// is used.
#[must_use]
pub fn resolve_endpoint() -> Cow<'static, str> {
    if let Ok(v) = std::env::var(ENDPOINT_ENV) {
        return Cow::Owned(v);
    }
    #[cfg(windows)]
    {
        Cow::Borrowed(DEFAULT_WINDOWS_ENDPOINT)
    }
    #[cfg(not(windows))]
    {
        Cow::Borrowed(DEFAULT_UNIX_ENDPOINT)
    }
}

// ── Test fixture with explicit poison handling ──────────────────────────────

static FIXTURE: Mutex<Option<InMemoryFixture>> = Mutex::new(None);
static NEXT_SUB_ID: Mutex<u64> = Mutex::new(1);

fn fixture_installed() -> Result<bool> {
    match FIXTURE.lock() {
        Ok(g) => Ok(g.is_some()),
        Err(_poisoned) => {
            // A previous panic left the fixture in an indeterminate state.
            // We refuse to proceed rather than silently recovering.
            Err(Error::TransportUnreachable(TransportFault::Internal))
        }
    }
}

fn next_subscription_id() -> Result<u64> {
    match NEXT_SUB_ID.lock() {
        Ok(mut g) => {
            let n = *g;
            *g = g.wrapping_add(1);
            Ok(n)
        }
        Err(_poisoned) => Err(Error::TransportUnreachable(TransportFault::Internal)),
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

    /// Install this fixture as the test backend.
    ///
    /// # Errors
    /// Returns [`TransportFault::Internal`] if the fixture mutex is poisoned.
    pub fn install(self) -> Result<()> {
        match FIXTURE.lock() {
            Ok(mut g) => {
                *g = Some(self);
                Ok(())
            }
            Err(_poisoned) => Err(Error::TransportUnreachable(TransportFault::Internal)),
        }
    }

    /// Remove the installed fixture.
    ///
    /// # Errors
    /// Returns [`TransportFault::Internal`] if the fixture mutex is poisoned.
    pub fn uninstall() -> Result<()> {
        match FIXTURE.lock() {
            Ok(mut g) => {
                *g = None;
                Ok(())
            }
            Err(_poisoned) => Err(Error::TransportUnreachable(TransportFault::Internal)),
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
    use crate::time::MonotonicTimestamp;
    use crate::{Capability, Direction, Manifest};
    use serial_test::serial;

    fn test_manifest() -> Manifest {
        Manifest::builder()
            .app_id("com.test.host")
            .unwrap()
            .capability(Capability::Navigation)
            .max_rate_hz(10)
            .build()
            .unwrap()
    }

    #[test]
    #[serial]
    fn connect_without_fixture_returns_transport_error() {
        let _ = InMemoryFixture::uninstall();
        let m = test_manifest();
        let r = connect_local(&m, StreamConfig::default());
        assert!(matches!(
            r,
            Err(Error::TransportUnreachable(
                TransportFault::EndpointNotFound
            ))
        ));
    }

    #[test]
    #[serial]
    fn connect_with_fixture_succeeds() {
        let mut fx = InMemoryFixture::new();
        let ts = MonotonicTimestamp::from_micros_unchecked(100);
        fx.push(IntentObservation::new_direction(
            ts,
            Direction::Right,
            45875,
            1,
            [0u8; 8],
        ));
        fx.install().unwrap();

        let m = test_manifest();
        let stream = connect_local(&m, StreamConfig::default()).unwrap();
        assert!(stream.is_connected());

        InMemoryFixture::uninstall().unwrap();
    }

    #[test]
    #[serial]
    fn endpoint_env_override_returns_owned_cow() {
        std::env::set_var(ENDPOINT_ENV, "/tmp/test.sock");
        let ep = resolve_endpoint();
        assert_eq!(&*ep, "/tmp/test.sock");
        assert!(matches!(ep, Cow::Owned(_)));
        std::env::remove_var(ENDPOINT_ENV);
    }

    #[test]
    #[serial]
    fn endpoint_default_returns_borrowed_cow() {
        std::env::remove_var(ENDPOINT_ENV);
        let ep = resolve_endpoint();
        assert!(matches!(ep, Cow::Borrowed(_)));
    }
}
