// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Application capabilities.
//!
//! # Formal specification
//!
//! A capability is a **grant** from the AxonOS kernel to an application,
//! authorizing it to receive a specific class of intent observations.
//!
//! ## Invariants
//! - Capabilities are **enumerated** — no custom capabilities exist.
//! - Capabilities are **immutable** after manifest handshake.
//! - Capabilities are **non-transferable** — an app cannot delegate
//!   its `Navigation` capability to another app.
//! - The kernel **verifies** capabilities against a hardware-backed
//!   policy store; the SDK merely declares intent.
//!
//! ## Wire format
//!
//! `CapabilitySet` is serialized as a **little-endian u32 bitfield**:
//! ```text
//! bit 0: Navigation
//! bit 1: WorkloadAdvisory
//! bit 2: SessionQuality
//! bit 3: ArtifactEvents
//! bits 4-31: reserved (must be zero)
//! ```
//!
//! Any set reserved bit causes `ManifestRejected::ProhibitedCapability`.

use core::fmt;

/// Application capability. Enumerated — no escape hatch by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Capability {
    /// Direction events for cursor/menu control. Kernel limit: 50 Hz.
    Navigation = 0,
    /// Cognitive load events. Kernel limit: 1 Hz.
    WorkloadAdvisory = 1,
    /// Signal-quality events. Kernel limit: 2 Hz.
    SessionQuality = 2,
    /// Artifact/electrode events. Kernel limit: 10 Hz.
    ArtifactEvents = 3,
}

impl Capability {
    /// Wire-level u8 discriminant.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Maximum events per second the kernel will deliver.
    #[must_use]
    pub const fn kernel_rate_limit_hz(self) -> u32 {
        match self {
            Self::Navigation => 50,
            Self::WorkloadAdvisory => 1,
            Self::SessionQuality => 2,
            Self::ArtifactEvents => 10,
        }
    }

    /// Human-readable name for audit logs.
    #[must_use]
    pub const fn audit_name(self) -> &'static str {
        match self {
            Self::Navigation => "navigation",
            Self::WorkloadAdvisory => "workload_advisory",
            Self::SessionQuality => "session_quality",
            Self::ArtifactEvents => "artifact_events",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.audit_name())
    }
}

// Compile-time guard: largest discriminant must fit in bitfield.
const _: () = {
    let max = Capability::ArtifactEvents as u8;
    assert!((max as usize) < (core::mem::size_of::<u32>() * 8));
};

/// Opaque raw representation of a [`CapabilitySet`].
///
/// The internal bit layout is not part of the stable public API.
/// Use this only for logging, metrics, or opaque storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawCapabilitySet(u32);

impl RawCapabilitySet {
    /// Raw u32 value. Stable within a major version.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Check if any reserved bits are set (invalid wire format).
    #[must_use]
    pub const fn has_reserved_bits(self) -> bool {
        self.0 & !0x0F != 0
    }
}

impl fmt::Display for RawCapabilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawCapabilitySet({:#010x})", self.0)
    }
}

/// A set of [`Capability`] values. Zero-allocation u32 bitfield.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CapabilitySet(u32);

impl CapabilitySet {
    /// Empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Add a capability.
    #[must_use]
    pub const fn with(mut self, c: Capability) -> Self {
        self.0 |= 1u32 << (c.as_u8() as u32);
        self
    }

    /// Check membership.
    #[must_use]
    pub const fn contains(&self, c: Capability) -> bool {
        (self.0 & (1u32 << (c.as_u8() as u32))) != 0
    }

    /// Count of capabilities in the set.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.0.count_ones()
    }

    /// True if empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Iterate over capabilities. Explicitly enumerated for exhaustiveness.
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

    /// Opaque raw representation.
    #[must_use]
    pub const fn as_raw(self) -> RawCapabilitySet {
        RawCapabilitySet(self.0)
    }

    /// Audit log format: human-readable list of capabilities.
    ///
    /// Example: `"[navigation, session_quality]"`
    pub fn audit_format(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        let mut first = true;
        for c in self.iter() {
            if !first {
                f.write_str(", ")?;
            }
            f.write_str(c.audit_name())?;
            first = false;
        }
        f.write_str("]")
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CapabilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.audit_format(f)
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
        assert!(!s.contains(Capability::WorkloadAdvisory));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn raw_is_opaque() {
        let s = CapabilitySet::new().with(Capability::Navigation);
        let raw = s.as_raw();
        assert_eq!(raw.as_u32(), 1);
        assert!(!raw.has_reserved_bits());
    }

    #[test]
    fn bitfield_width_accommodates_all_variants() {
        let s = CapabilitySet::new()
            .with(Capability::Navigation)
            .with(Capability::WorkloadAdvisory)
            .with(Capability::SessionQuality)
            .with(Capability::ArtifactEvents);
        assert_eq!(s.as_raw().as_u32(), 0x0F);
    }

    #[test]
    fn display_format() {
        let s = CapabilitySet::new()
            .with(Capability::Navigation)
            .with(Capability::SessionQuality);
        assert_eq!(format!("{}", s), "[navigation, session_quality]");
    }

    #[test]
    fn audit_name_matches_display() {
        assert_eq!(Capability::Navigation.audit_name(), "navigation");
        assert_eq!(format!("{}", Capability::Navigation), "navigation");
    }
}
