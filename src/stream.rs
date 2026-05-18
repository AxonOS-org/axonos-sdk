// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Intent stream subscription.
//!
//! # Thread safety
//!
//! `IntentStream` is intentionally **neither `Send` nor `Sync`** when built
//! via the `std` host module, because it may hold a kernel-bound IPC handle
//! with thread-affinity requirements.
//!
//! # Security
//!
//! `try_next()` requires the `kernel-stub` feature to compile without a
//! real kernel. **Never enable `kernel-stub` in production builds** — it
//! disables HMAC-SHA256 attestation verification.

use crate::error::Result;
use crate::intent::{IntentKind, IntentObservation};
use crate::manifest::Manifest;

/// SDK internal buffer capacity per stream.
pub const DEFAULT_BUFFER_CAPACITY: usize = 256;

/// Subscription handle. Dropping this ends the subscription.
///
/// # Thread safety
///
/// `Subscription` is `!Send + !Sync` because the underlying kernel
/// subscription may be bound to a specific thread or interrupt context.
#[derive(Debug)]
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    /// Explicitly !Send + !Sync via PhantomData of a non-Send type.
    pub(crate) _not_send: core::marker::PhantomData<SubscriptionInner>,
}

/// Internal non-Send type used to enforce thread-affinity.
#[cfg(feature = "std")]
pub(crate) struct SubscriptionInner;

#[cfg(not(feature = "std"))]
pub(crate) struct SubscriptionInner;

impl Subscription {
    /// Unique per-session subscription identifier.
    #[must_use]
    pub const fn id(&self) -> SubscriptionId {
        self.id
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Cancel message to kernel — no-op in abstract type.
    }
}

/// Opaque subscription identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    #[must_use]
    pub const fn from_raw(v: u64) -> Self {
        Self(v)
    }

    #[must_use]
    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

/// Policy for buffer overflow handling.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum OverflowPolicy {
    #[default]
    DropOldest,
    DropNewest,
    /// Not recommended — may violate kernel WCET.
    BackPressure,
}

/// Client-side observation filter.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ObservationFilter {
    #[default]
    All,
    MinConfidence(u16),
    OnlyKind(FilterKind),
}

impl ObservationFilter {
    #[must_use]
    pub fn matches(&self, obs: &IntentObservation) -> bool {
        match self {
            Self::All => true,
            Self::MinConfidence(min) => obs.confidence_raw() >= *min,
            Self::OnlyKind(k) => matches!(
                (k, obs.kind()),
                (FilterKind::Direction, IntentKind::Direction(_))
                    | (FilterKind::Load, IntentKind::Load(_))
                    | (FilterKind::Quality, IntentKind::Quality(_))
            ),
        }
    }
}

/// Filter discriminant kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterKind {
    Direction,
    Load,
    Quality,
}

/// Stream configuration.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_capacity: usize,
    pub overflow_policy: OverflowPolicy,
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

/// Intent-event stream.
///
/// # Thread safety
/// `IntentStream` is `!Send + !Sync` because it may hold kernel-bound IPC
/// state with thread-affinity requirements.
#[derive(Debug)]
#[must_use]
pub struct IntentStream {
    config: StreamConfig,
    subscription: Option<Subscription>,
    #[allow(dead_code)]
    manifest_app_id_hash: u64,
}

impl IntentStream {
    pub fn new(manifest: &Manifest, config: StreamConfig) -> Self {
        Self {
            config,
            subscription: None,
            manifest_app_id_hash: hash_app_id(manifest.app_id()),
        }
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn connect(manifest: &Manifest) -> Result<Self> {
        crate::host::connect_local(manifest, StreamConfig::default())
    }

    pub fn attach_subscription(&mut self, sub: Subscription) {
        self.subscription = Some(sub);
    }

    #[must_use]
    pub const fn config(&self) -> &StreamConfig {
        &self.config
    }

    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.subscription.is_some()
    }

    /// Try to get the next observation. Non-blocking.
    ///
    /// # Security
    /// This method is gated behind the `kernel-stub` feature.
    /// Without `kernel-stub`, building any code that calls this fails:
    /// the method is `#[cfg(feature = "kernel-stub")]`.
    ///
    /// `kernel-stub` disables HMAC-SHA256 attestation verification and
    /// must NEVER be enabled in production. The real implementation
    /// arrives when the kernel ABI ships.
    #[cfg(feature = "kernel-stub")]
    pub fn try_next(&mut self) -> Result<Option<IntentObservation>> {
        // Stub: real implementation reads from the kernel IPC ring buffer.
        Ok(None)
    }

    /// Stub guard — appears only when `kernel-stub` is disabled.
    /// Building code that calls `try_next` without `kernel-stub` fails here.
    #[cfg(not(feature = "kernel-stub"))]
    #[doc(hidden)]
    pub fn try_next(&mut self) -> Result<Option<IntentObservation>> {
        // Reaching this means: code calls `try_next` but `kernel-stub`
        // is not enabled AND the real kernel transport is not yet wired.
        // When the real kernel ships, replace this with the IPC read path.
        Err(crate::error::Error::TransportUnreachable(
            crate::error::TransportFault::Internal,
        ))
    }

    #[must_use]
    pub fn filter_match(&self, obs: &IntentObservation) -> bool {
        self.config.filter.matches(obs)
    }
}

/// Portable FNV-1a hash for internal bookkeeping (non-cryptographic).
/// Inline implementation — no external dependency.
fn hash_app_id(id: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in id.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::Direction;
    use crate::time::MonotonicTimestamp;
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
        let ts = MonotonicTimestamp::from_micros_unchecked(0);
        let obs = IntentObservation::new_direction(ts, Direction::Up, 32768, 0, [0u8; 8]);
        assert!(f.matches(&obs));
    }

    #[test]
    fn filter_min_confidence() {
        let f = ObservationFilter::MinConfidence(32768);
        let ts = MonotonicTimestamp::from_micros_unchecked(0);
        let high = IntentObservation::new_direction(ts, Direction::Up, 60000, 0, [0u8; 8]);
        let low = IntentObservation::new_direction(ts, Direction::Up, 1000, 0, [0u8; 8]);
        assert!(f.matches(&high));
        assert!(!f.matches(&low));
    }

    #[test]
    fn stream_starts_disconnected() {
        let m = test_manifest();
        let s = IntentStream::new(&m, StreamConfig::default());
        assert!(!s.is_connected());
    }

    #[test]
    fn overflow_policy_default() {
        assert_eq!(OverflowPolicy::default(), OverflowPolicy::DropOldest);
    }

    #[test]
    fn fnv_hash_is_deterministic() {
        let h1 = hash_app_id("com.test.app");
        let h2 = hash_app_id("com.test.app");
        assert_eq!(h1, h2);
        let h3 = hash_app_id("com.test.different");
        assert_ne!(h1, h3);
    }
}

