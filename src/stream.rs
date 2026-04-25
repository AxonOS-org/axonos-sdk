// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Intent stream subscription.
//!
//! [`IntentStream`] is the primary API applications use to receive intent
//! observations from the AxonOS kernel. It is transport-agnostic — the
//! same type works over a local IPC endpoint, a test fixture, or a shared
//! memory ring buffer.
//!
//! # Filters
//!
//! Applications can optionally install an [`ObservationFilter`] to reduce
//! delivery to a subset of events. Filtering happens on the **application
//! side** in this SDK — the kernel still delivers every event. This trades
//! bandwidth for simplicity; applications that need server-side filtering
//! must register specific capability subsets in their manifest instead.
//!
//! # Overflow
//!
//! If the application cannot drain the stream fast enough, the kernel will
//! drop observations according to the configured [`OverflowPolicy`]. The
//! application is notified via [`crate::Error::StreamOverflow`] on the
//! next `next()` call.

use crate::error::Result;
use crate::intent::{IntentKind, IntentObservation};
use crate::manifest::Manifest;

/// Maximum number of in-flight observations the SDK buffers internally,
/// per stream. This is independent of the kernel ring buffer size.
pub const DEFAULT_BUFFER_CAPACITY: usize = 256;

/// Subscription handle. Dropping this ends the subscription.
#[derive(Debug)]
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    /// Kept to ensure the stream is closed when the subscription is dropped.
    pub(crate) _phantom: core::marker::PhantomData<*const ()>,
}

impl Subscription {
    /// Unique per-session subscription identifier.
    #[must_use]
    pub const fn id(&self) -> SubscriptionId {
        self.id
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // In a real implementation this would send a cancel message to the
        // kernel; in the SDK abstract type it is a no-op. The host module
        // contains the concrete implementation.
    }
}

/// Opaque subscription identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Construct from a raw u64 (used by test fixtures and the host module).
    #[must_use]
    pub const fn from_raw(v: u64) -> Self {
        Self(v)
    }

    /// Raw value.
    #[must_use]
    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

/// Policy for what the kernel does when the application cannot drain the
/// stream fast enough.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum OverflowPolicy {
    /// Drop the oldest observations. Suitable for continuous-control
    /// applications where fresh state matters more than history.
    DropOldest,
    /// Drop the newest observations. Suitable for applications that cannot
    /// accept out-of-order delivery.
    DropNewest,
    /// Do not drop — instead, back-pressure the kernel. **Not recommended**
    /// for AxonOS, because it can cause pipeline stalls that violate the
    /// kernel's WCET contract. The kernel may unilaterally switch to
    /// `DropOldest` after a configurable threshold.
    BackPressure,
}

impl Default for OverflowPolicy {
    fn default() -> Self {
        Self::DropOldest
    }
}

/// A predicate that filters incoming observations. Filtering happens
/// client-side; events that don't match are discarded without being
/// returned to the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationFilter {
    /// Deliver all observations.
    All,
    /// Deliver only observations whose confidence is ≥ the given threshold.
    MinConfidence(u16),
    /// Deliver only observations of the given discriminant.
    OnlyKind(FilterKind),
}

impl ObservationFilter {
    /// Check whether an observation passes this filter.
    #[must_use]
    pub fn matches(&self, obs: &IntentObservation) -> bool {
        match self {
            Self::All => true,
            Self::MinConfidence(min) => obs.quality_raw >= *min,
            Self::OnlyKind(k) => match (k, obs.kind()) {
                (FilterKind::Direction, IntentKind::Direction(_))
                | (FilterKind::Load, IntentKind::Load(_))
                | (FilterKind::Quality, IntentKind::Quality(_)) => true,
                _ => false,
            },
        }
    }
}

impl Default for ObservationFilter {
    fn default() -> Self {
        Self::All
    }
}

/// Discriminant kinds, for use with [`ObservationFilter::OnlyKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterKind {
    /// Direction events only.
    Direction,
    /// Load events only.
    Load,
    /// Quality events only.
    Quality,
}

/// Configuration for a new [`IntentStream`].
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Client-side buffer depth.
    pub buffer_capacity: usize,
    /// What to do when the buffer overflows.
    pub overflow_policy: OverflowPolicy,
    /// Client-side filter.
    pub filter: ObservationFilter,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: DEFAULT_BUFFER_CAPACITY,
            overflow_policy: OverflowPolicy::default(),
            filter: ObservationFilter::default(),
        }
    }
}

/// An intent-event stream.
///
/// # Note on transport
///
/// This module provides the abstract type. The concrete connection logic
/// (IPC, shared memory, test fixtures) lives in [`crate::host`] when the
/// `std` feature is enabled. For no_std builds, applications integrate
/// directly with the kernel ring buffer — see the `bare_metal_no_std`
/// example.
#[derive(Debug)]
#[must_use]
pub struct IntentStream {
    config: StreamConfig,
    subscription: Option<Subscription>,
    /// Retained for future IPC correlation; currently unused.
    #[allow(dead_code)]
    manifest_app_id_hash: u64,
}

impl IntentStream {
    /// Create a new stream with the given configuration, bound to the given
    /// manifest.
    ///
    /// In a real application, this handshakes with the kernel; in this SDK
    /// it returns an unconnected handle. Use [`IntentStream::connect`] (std
    /// feature) for the full connection flow.
    #[must_use]
    pub fn new(manifest: &Manifest, config: StreamConfig) -> Self {
        Self {
            config,
            subscription: None,
            manifest_app_id_hash: hash_app_id(manifest.app_id()),
        }
    }

    /// Connect to the local AxonOS kernel endpoint with default configuration.
    ///
    /// This is an abbreviated constructor; use [`crate::host::connect_local`]
    /// if you need a custom [`StreamConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TransportUnreachable`] if the kernel IPC endpoint
    /// is not available, or [`Error::ManifestRejected`] if the manifest
    /// fails kernel-side validation.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn connect(manifest: &Manifest) -> Result<Self> {
        crate::host::connect_local(manifest, StreamConfig::default())
    }

    /// Associate a subscription with this stream (usually done by the host
    /// module after a successful handshake).
    pub fn attach_subscription(&mut self, sub: Subscription) {
        self.subscription = Some(sub);
    }

    /// Current configuration.
    #[must_use]
    pub const fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Whether the stream is connected to a subscription.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.subscription.is_some()
    }

    /// Try to get the next observation. Non-blocking.
    ///
    /// # Errors
    ///
    /// Returns the same error set as [`crate::Error`]. In particular,
    /// [`Error::ConsentSuspended`] is non-terminal — the caller should
    /// retry after a consent-resume event.
    pub fn try_next(&mut self) -> Result<Option<IntentObservation>> {
        // The host module implements the real I/O. Here we return None
        // to indicate "no observation available" — a fully working
        // implementation is provided in the feature-gated host module.
        Ok(None)
    }

    /// Apply the configured filter to an observation. Exposed for testing
    /// and for applications that want to reuse the filter logic.
    #[must_use]
    pub fn filter_match(&self, obs: &IntentObservation) -> bool {
        self.config.filter.matches(obs)
    }
}

/// Hash the app_id for internal bookkeeping (stable, non-cryptographic).
fn hash_app_id(id: &str) -> u64 {
    use core::hash::Hasher;
    let mut h = siphasher::sip::SipHasher::new();
    h.write(id.as_bytes());
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Direction;
    use crate::{Capability, Manifest};

    fn test_manifest() -> Manifest {
        Manifest::builder()
            .app_id("com.test.a")
            .unwrap()
            .capability(Capability::Navigation)
            .max_rate_hz(10)
            .build()
            .unwrap()
    }

    #[test]
    fn filter_all_matches_everything() {
        let f = ObservationFilter::All;
        let obs = IntentObservation::new_direction(0, Direction::Up, 0.5, 0, [0u8; 8]);
        assert!(f.matches(&obs));
    }

    #[test]
    fn filter_min_confidence() {
        let f = ObservationFilter::MinConfidence(u16::MAX / 2); // ~0.5
        let high = IntentObservation::new_direction(0, Direction::Up, 0.9, 0, [0u8; 8]);
        let low = IntentObservation::new_direction(0, Direction::Up, 0.1, 0, [0u8; 8]);
        assert!(f.matches(&high));
        assert!(!f.matches(&low));
    }

    #[test]
    fn filter_only_kind() {
        let f = ObservationFilter::OnlyKind(FilterKind::Direction);
        let dir = IntentObservation::new_direction(0, Direction::Up, 0.5, 0, [0u8; 8]);
        let q = IntentObservation::new_quality(0, crate::Quality::High, 0, [0u8; 8]);
        assert!(f.matches(&dir));
        assert!(!f.matches(&q));
    }

    #[test]
    fn stream_starts_disconnected() {
        let m = test_manifest();
        let s = IntentStream::new(&m, StreamConfig::default());
        assert!(!s.is_connected());
        assert_eq!(s.config().buffer_capacity, DEFAULT_BUFFER_CAPACITY);
    }

    #[test]
    fn overflow_policy_default() {
        assert_eq!(OverflowPolicy::default(), OverflowPolicy::DropOldest);
    }
}
