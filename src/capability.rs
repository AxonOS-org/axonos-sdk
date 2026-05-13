// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Application capabilities.
//!
//! # Formal specification
//!
//! A capability is a grant from the AxonOS kernel to an application,
//! authorizing it to receive a specific class of intent observations.
//!
//! ## Invariants
//! - Capabilities are enumerated --- no custom capabilities exist.
//! - Capabilities are immutable after manifest handshake.
//! - Capabilities are non-transferable.
//! - The kernel verifies capabilities against a hardware-backed policy
//!   store; the SDK merely declares intent.
//!
//! ## Wire format
//!
//! `CapabilitySet` is serialized as a little-endian u32 bitfield:
//! ```text
//! bit 0: Navigation
//! bit 1: WorkloadAdvisory
//! bit 2: SessionQuality
//! bit 3: ArtifactEvents
//! bits 4..31: reserved (must be zero)
//! ```
//! Any set reserved bit causes `ManifestRejected::ProhibitedCapability`.

use core::fmt;

/// Application capability. Enumerated --- no escape hatch by design.
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

/// Total number of [`Capability`] variants.
///
/// Single source of truth for the bitfield-width invariant.
pub const CAPABILITY_COUNT: u8 = 4;

impl Capability {
    /// Wire-level u8 discriminant.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Maximum events per second the kernel will deliver for this capability.
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

    /// Construct from u8 discriminant.
    ///
    /// Returns `None` if the value does not correspond to a known capability,
    /// preventing the construction of out-of-range values from external input.
    #[must_use]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Navigation),
            1 => Some(Self::WorkloadAdvisory),
            2 => Some(Self::SessionQuality),
            3 => Some(Self::ArtifactEvents),
            _ => None,
        }
    }

    /// Bit position in the [`CapabilitySet`] bitfield.
    ///
    /// Centralised so the `with`/`contains` operations share one source of
    /// truth and the compile-time guard does not need to be duplicated.
    #[inline]
    const fn bit(self) -> u32 {
        // Safe: `as_u8() < CAPABILITY_COUNT < 32` (guard below).
        1u32.wrapping_shl(self.as_u8() as u32)
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.audit_name())
    }
}

// Compile-time guard: all variants fit in the u32 bitfield.
// If a new variant is added past index 31, this fails at build time.
const _: () = {
    assert!((CAPABILITY_COUNT as usize) < (core::mem::size_of::<u32>() * 8));
};

/// Opaque raw representation of a [`CapabilitySet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawCapabilitySet(u32);

impl RawCapabilitySet {
    /// Raw u32 value. Stable within a major version.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// True if any reserved bits are set (invalid wire format).
    #[must_use]
    pub const fn has_reserved_bits(self) -> bool {
        // Reserved mask: bits 4..31 must be zero.
        let valid_mask: u32 = (1u32.wrapping_shl(CAPABILITY_COUNT as u32)) - 1;
        self.0 & !valid_mask != 0
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
    ///
    /// Uses `wrapping_shl` for defence-in-depth: even if `CAPABILITY_COUNT`
    /// is incorrectly raised past 32 without updating the compile-time
    /// guard, this operation will not invoke undefined behaviour.
    #[must_use]
    pub const fn with(mut self, c: Capability) -> Self {
        self.0 |= c.bit();
        self
    }

    /// Check membership.
    #[must_use]
    pub const fn contains(&self, c: Capability) -> bool {
        (self.0 & c.bit()) != 0
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

    /// Iterate over capabilities in discriminant order.
    ///
    /// Zero-cost custom iterator (4-state machine, no array allocation).
    #[must_use]
    pub const fn iter(&self) -> CapabilityIter {
        CapabilityIter { bits: self.0, idx: 0 }
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

/// Zero-cost iterator over capabilities in a [`CapabilitySet`].
///
/// Yields capabilities in discriminant order (Navigation, WorkloadAdvisory,
/// SessionQuality, ArtifactEvents).
#[derive(Debug, Clone)]
pub struct CapabilityIter {
    bits: u32,
    idx: u8,
}

impl Iterator for CapabilityIter {
    type Item = Capability;

    fn next(&mut self) -> Option<Capability> {
        while self.idx < CAPABILITY_COUNT {
            let bit = 1u32.wrapping_shl(self.idx as u32);
            let current = self.idx;
            self.idx += 1;
            if self.bits & bit != 0 {
                return Capability::from_u8(current);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.bits & !((1u32 << self.idx).wrapping_sub(1))).count_ones() as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for CapabilityIter {}

impl IntoIterator for CapabilitySet {
    type Item = Capability;
    type IntoIter = CapabilityIter;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a CapabilitySet {
    type Item = Capability;
    type IntoIter = CapabilityIter;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
    fn reserved_bits_detected() {
        let raw = RawCapabilitySet(0xFF00_0000);
        assert!(raw.has_reserved_bits());

        let raw_ok = RawCapabilitySet(0x0F);
        assert!(!raw_ok.has_reserved_bits());
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

    #[test]
    fn from_u8_rejects_out_of_range() {
        assert_eq!(Capability::from_u8(0), Some(Capability::Navigation));
        assert_eq!(Capability::from_u8(3), Some(Capability::ArtifactEvents));
        assert_eq!(Capability::from_u8(4), None);
        assert_eq!(Capability::from_u8(255), None);
    }

    #[test]
    fn into_iterator_owned() {
        let s = CapabilitySet::new()
            .with(Capability::Navigation)
            .with(Capability::ArtifactEvents);
        let collected: heapless::Vec<Capability, 4> = s.into_iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], Capability::Navigation);
        assert_eq!(collected[1], Capability::ArtifactEvents);
    }

    #[test]
    fn into_iterator_borrowed() {
        let s = CapabilitySet::new().with(Capability::SessionQuality);
        let mut count = 0;
        for c in &s {
            assert_eq!(c, Capability::SessionQuality);
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn iter_yields_in_discriminant_order() {
        let s = CapabilitySet::new()
            .with(Capability::ArtifactEvents)
            .with(Capability::Navigation);
        let collected: heapless::Vec<Capability, 4> = s.iter().collect();
        // Navigation (bit 0) before ArtifactEvents (bit 3)
        assert_eq!(collected[0], Capability::Navigation);
        assert_eq!(collected[1], Capability::ArtifactEvents);
    }

    #[test]
    fn iter_exact_size() {
        let s = CapabilitySet::new()
            .with(Capability::Navigation)
            .with(Capability::SessionQuality);
        assert_eq!(s.iter().len(), 2);
    }
}
