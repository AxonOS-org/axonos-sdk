// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Application capabilities.
//!
//! Every AxonOS application must declare a [`CapabilitySet`] in its
//! [`crate::Manifest`]. Capabilities describe the **classes of intent
//! observations** the application is authorized to receive.
//!
//! # Permitted capabilities
//!
//! Only capabilities in the [`Capability`] enum can be declared. The enum
//! is exhaustive by design — there is no "custom capability" escape hatch,
//! because the capability surface is an IEC 62304 traceable risk-control
//! boundary.
//!
//! # Prohibited capabilities (not in the enum)
//!
//! The following capability classes are **deliberately absent** from this
//! enum and will be rejected by the kernel manifest verifier:
//!
//! - Raw EEG access — available only to `axonos-kernel` internals.
//! - Continuous emotion inference — prohibited per AxonOS neuroethics policy.
//! - Cognitive profile read — prohibited per the same policy.
//! - Re-identification — prohibited per UNESCO Recommendation on the Ethics
//!   of Neurotechnology (2025), §III.
//!
//! Attempting to construct a manifest targeting any of these categories
//! returns [`crate::Error::ManifestRejected`] with reason
//! [`crate::error::ManifestRejection::ProhibitedCapability`].

use core::fmt;

/// An application capability. Each variant corresponds to a class of
/// intent observations the application is authorized to consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Capability {
    /// Receive [`crate::IntentKind::Direction`] events. Typical use:
    /// cursor control, menu navigation.
    Navigation = 0,

    /// Receive [`crate::IntentKind::Load`] events. Typical use:
    /// adaptive UI that simplifies when user is under high cognitive load.
    /// Rate-limited by kernel policy to ≤1 Hz.
    WorkloadAdvisory = 1,

    /// Receive [`crate::IntentKind::Quality`] events. Typical use:
    /// show the user a signal-quality indicator.
    SessionQuality = 2,

    /// Receive artifact / electrode-event notifications (e.g., "user blinked"
    /// debiased). Typical use: calibration UX.
    ArtifactEvents = 3,
}

impl Capability {
    /// Returns the capability as its wire-level `u8` discriminant.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Maximum events per second the kernel will deliver for this capability.
    /// Applications can declare a lower rate; they cannot exceed this.
    #[must_use]
    pub const fn kernel_rate_limit_hz(self) -> u32 {
        match self {
            Self::Navigation => 50,
            Self::WorkloadAdvisory => 1,
            Self::SessionQuality => 2,
            Self::ArtifactEvents => 10,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Navigation => f.write_str("navigation"),
            Self::WorkloadAdvisory => f.write_str("workload_advisory"),
            Self::SessionQuality => f.write_str("session_quality"),
            Self::ArtifactEvents => f.write_str("artifact_events"),
        }
    }
}

/// A set of [`Capability`] values. Implemented as a `u8` bitfield for
/// zero-allocation storage in [`crate::Manifest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CapabilitySet(u8);

impl CapabilitySet {
    /// Empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Add a capability. Returns `self` for chaining.
    #[must_use]
    pub const fn with(mut self, c: Capability) -> Self {
        self.0 |= 1 << c.as_u8();
        self
    }

    /// Check whether the set contains a capability.
    #[must_use]
    pub const fn contains(&self, c: Capability) -> bool {
        (self.0 & (1 << c.as_u8())) != 0
    }

    /// Number of capabilities in the set.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.0.count_ones()
    }

    /// `true` if the set is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Iterate over the capabilities in the set.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        [
            Capability::Navigation,
            Capability::WorkloadAdvisory,
            Capability::SessionQuality,
            Capability::ArtifactEvents,
        ]
        .into_iter()
        .filter(|c| self.contains(*c))
    }

    /// Raw bitfield representation. Stable across SDK versions within the
    /// same major-version series.
    #[must_use]
    pub const fn as_raw(self) -> u8 {
        self.0
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_round_trip() {
        let s = CapabilitySet::new()
            .with(Capability::Navigation)
            .with(Capability::SessionQuality);
        assert!(s.contains(Capability::Navigation));
        assert!(s.contains(Capability::SessionQuality));
        assert!(!s.contains(Capability::WorkloadAdvisory));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn set_iter_preserves_order() {
        let s = CapabilitySet::new()
            .with(Capability::ArtifactEvents)
            .with(Capability::Navigation);
        let collected: heapless::Vec<Capability, 4> = s.iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], Capability::Navigation);
        assert_eq!(collected[1], Capability::ArtifactEvents);
    }

    #[test]
    fn empty_set() {
        let s = CapabilitySet::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert!(!s.contains(Capability::Navigation));
    }

    #[test]
    fn rate_limits_are_documented() {
        // These numbers are part of the public contract.
        assert_eq!(Capability::Navigation.kernel_rate_limit_hz(), 50);
        assert_eq!(Capability::WorkloadAdvisory.kernel_rate_limit_hz(), 1);
    }
}
