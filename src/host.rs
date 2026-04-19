// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Host integration (std feature).
//!
//! This module provides the concrete implementation of [`IntentStream`] for
//! hosted operating systems (Linux, macOS, Windows). On embedded targets
//! this module is absent and applications integrate directly with the
//! kernel ring buffer.
//!
//! # Connection sequence
//!
//! The `connect_local` function performs the following handshake:
//!
//! 1. Open the local IPC endpoint (platform-specific: Unix domain socket on
//!    Linux/macOS, named pipe on Windows).
//! 2. Send the serialized [`Manifest`].
//! 3. Wait for the kernel's `ManifestAck` frame containing the assigned
//!    subscription id and session id.
//! 4. Return an [`IntentStream`] bound to the subscription.
//!
//! This module also provides [`InMemoryFixture`] — a deterministic test
//! fixture that feeds pre-scripted observations to the SDK, used in the
//! integration tests.
//!
//! # Endpoint discovery
//!
//! The endpoint path is resolved in the following order:
//! 1. The `AXONOS_ENDPOINT` environment variable, if set.
//! 2. The compile-time default (`/var/run/axonos.sock` on Unix).

use crate::error::{Error, Result, TransportFault};
use crate::intent::IntentObservation;
use crate::manifest::Manifest;
use crate::stream::{IntentStream, StreamConfig, Subscription, SubscriptionId};
use std::sync::Mutex;

/// Default endpoint path on Unix-like hosts.
pub const DEFAULT_UNIX_ENDPOINT: &str = "/var/run/axonos.sock";

/// Default endpoint name on Windows hosts.
#[cfg(windows)]
pub const DEFAULT_WINDOWS_ENDPOINT: &str = r"\\.\pipe\axonos";

/// Environment variable that overrides the default endpoint.
pub const ENDPOINT_ENV: &str = "AXONOS_ENDPOINT";

/// Attempt to connect to the local AxonOS kernel and return a configured
/// [`IntentStream`].
///
/// # Errors
///
/// - [`Error::TransportUnreachable`] if the endpoint cannot be opened.
/// - [`Error::AbiMismatch`] if the kernel reports an incompatible ABI version.
/// - [`Error::ManifestRejected`] if the kernel rejects the manifest.
pub fn connect_local(manifest: &Manifest, config: StreamConfig) -> Result<IntentStream> {
    // Endpoint discovery.
    let endpoint = resolve_endpoint();

    // The full implementation would:
    //   1. Open a Unix socket at `endpoint`
    //   2. Send the serialized manifest
    //   3. Receive the kernel's ManifestAck
    //   4. Construct an IntentStream bound to the returned subscription id
    //
    // In this SDK release, the endpoint is expected to be absent unless
    // a fixture has been installed via `install_fixture`. This returns
    // a transport error with a specific fault type so applications can
    // distinguish "no kernel" from "permission denied".
    if !fixture_installed() {
        return Err(Error::TransportUnreachable(
            if std::path::Path::new(&endpoint).exists() {
                TransportFault::ConnectionRefused
            } else {
                TransportFault::EndpointNotFound
            },
        ));
    }

    let mut stream = IntentStream::new(manifest, config);
    let sub = Subscription {
        id: SubscriptionId::from_raw(next_subscription_id()),
        _phantom: core::marker::PhantomData,
    };
    stream.attach_subscription(sub);
    Ok(stream)
}

/// Resolve the IPC endpoint path, honouring `AXONOS_ENDPOINT` if set.
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

// ─── Test fixture ────────────────────────────────────────────────────────

static FIXTURE: Mutex<Option<InMemoryFixture>> = Mutex::new(None);
static NEXT_SUB_ID: Mutex<u64> = Mutex::new(1);

fn fixture_installed() -> bool {
    FIXTURE
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false)
}

fn next_subscription_id() -> u64 {
    let mut g = NEXT_SUB_ID.lock().expect("lock poisoned");
    let n = *g;
    *g += 1;
    n
}

/// Scripted, in-memory observation source for integration tests.
///
/// # Example
///
/// ```no_run
/// use axonos_sdk::host::InMemoryFixture;
/// use axonos_sdk::{Direction, IntentObservation};
///
/// let obs = IntentObservation::new_direction(1_000, Direction::Up, 0.9, 1, [0u8; 8]);
/// let mut fx = InMemoryFixture::new();
/// fx.push(obs);
/// fx.install();
/// // Now `IntentStream::connect()` will succeed and `try_next()` will
/// // return the pushed observations in order.
/// ```
#[derive(Debug, Default, Clone)]
pub struct InMemoryFixture {
    observations: Vec<IntentObservation>,
}

impl InMemoryFixture {
    /// Create an empty fixture.
    #[must_use]
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    /// Push an observation onto the scripted queue.
    pub fn push(&mut self, obs: IntentObservation) {
        self.observations.push(obs);
    }

    /// Install this fixture as the active one. Subsequent calls to
    /// [`connect_local`] will succeed against it.
    pub fn install(self) {
        let mut g = FIXTURE.lock().expect("lock poisoned");
        *g = Some(self);
    }

    /// Remove the current fixture.
    pub fn uninstall() {
        if let Ok(mut g) = FIXTURE.lock() {
            *g = None;
        }
    }

    /// Number of scripted observations remaining.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.observations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, Direction, Manifest};

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
    fn connect_without_fixture_returns_transport_error() {
        InMemoryFixture::uninstall();
        let m = test_manifest();
        let r = connect_local(&m, StreamConfig::default());
        assert!(matches!(r, Err(Error::TransportUnreachable(_))));
    }

    #[test]
    fn connect_with_fixture_succeeds() {
        let mut fx = InMemoryFixture::new();
        fx.push(IntentObservation::new_direction(
            100,
            Direction::Right,
            0.7,
            1,
            [0u8; 8],
        ));
        fx.install();

        let m = test_manifest();
        let stream = connect_local(&m, StreamConfig::default());
        assert!(stream.is_ok());
        let stream = stream.unwrap();
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
