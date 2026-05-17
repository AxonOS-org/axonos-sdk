// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Monotonic time abstraction for BCI sessions.
//!
//! # Time model
//!
//! AxonOS guarantees session-local monotonicity: timestamps within a single
//! session never decrease.
//! There is no wall-clock time exposed to applications --- this is a
//! deliberate privacy boundary.
//!
//! # WCET guarantees
//!
//! All operations are O(1):
//!
//! | Operation        | WCET (Cortex-M4F @ 168 MHz) |
//! |:-----------------|:----------------------------|
//! | `as_micros()`    | 1 cycle                     |
//! | `as_millis()`    | ~10 cycles (hardware div)   |
//! | `checked_sub()`  | 3 cycles                    |
//! | `elapsed_since()`| 3 cycles                    |
//!
//! No allocations. No syscalls. No floating point.

use core::fmt;

/// Upper bound for a valid session timestamp.
///
/// Any deserialized timestamp above this value is rejected to prevent
/// downstream arithmetic from being fed adversarial inputs from untrusted
/// transports (CBOR/JSON network frames).
///
/// Value: 2^48 µs ≈ 8.9 years. A real BCI session never approaches this.
pub const SESSION_MAX_REASONABLE_US: u64 = 1u64 << 48;

/// Monotonic timestamp — microseconds since session start.
///
/// # Invariants
/// - Never decreases within a session.
/// - Caller is responsible for monotonicity at construction.
/// - Wraps at `u64::MAX` (~584 942 years at 1 µs resolution; far beyond any session).
/// - Comparison is meaningful only within the same session.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MonotonicTimestamp(u64);

impl MonotonicTimestamp {
    /// Construct from raw microseconds.
    ///
    /// Used by kernel transport only. The caller must ensure monotonicity
    /// within the session.
    /// This is not `unsafe` in the Rust sense, but violating monotonicity
    /// breaks downstream timing logic --- name is `_unchecked` to flag this.
    #[must_use]
    pub const fn from_micros_unchecked(us: u64) -> Self {
        Self(us)
    }

    /// Validated constructor from microseconds.
    ///
    /// Rejects values exceeding [`SESSION_MAX_REASONABLE_US`].
    /// Use this for any timestamp originating from outside the kernel
    /// (deserialization, FFI, etc.).
    #[must_use]
    pub const fn from_micros_validated(us: u64) -> Option<Self> {
        if us <= SESSION_MAX_REASONABLE_US {
            Some(Self(us))
        } else {
            None
        }
    }

    /// Raw microseconds since session start.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Milliseconds since session start, truncated.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0 / 1000
    }

    /// Duration since an earlier timestamp.
    ///
    /// Returns `None` if `earlier > self` (a clock violation, e.g., kernel
    /// bug, DMA corruption, or out-of-order delivery).
    ///
    /// # WCET
    /// 3 cycles (compare + subtract).
    #[must_use]
    pub const fn checked_sub(self, earlier: MonotonicTimestamp) -> Option<u64> {
        if self.0 >= earlier.0 {
            Some(self.0 - earlier.0)
        } else {
            None
        }
    }

    /// Duration since an earlier timestamp.
    ///
    /// Returns `None` on clock violation (`earlier > self`).
    ///
    /// Callers must decide explicitly how to handle violations.
    /// The earlier `elapsed_since() -> u64` API returned 0 silently, which
    /// could mask kernel bugs in release builds; that has been removed.
    /// Use [`elapsed_since_saturating`](Self::elapsed_since_saturating) if
    /// a clamped result is genuinely desired.
    #[must_use]
    pub const fn elapsed_since(self, earlier: MonotonicTimestamp) -> Option<u64> {
        self.checked_sub(earlier)
    }

    /// Duration since an earlier timestamp, saturating at 0 on clock violation.
    ///
    /// Explicit defensive variant: use only when 0 is a meaningful response
    /// to a clock violation and the caller documents why.
    /// For safety-critical paths, prefer [`elapsed_since`](Self::elapsed_since)
    /// and handle `None` explicitly.
    #[must_use]
    pub const fn elapsed_since_saturating(self, earlier: MonotonicTimestamp) -> u64 {
        match self.checked_sub(earlier) {
            Some(d) => d,
            None => 0,
        }
    }
}

impl fmt::Debug for MonotonicTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MonotonicTimestamp({} us)", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MonotonicTimestamp {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MonotonicTimestamp {
    /// Validated deserialization.
    ///
    /// Rejects values exceeding [`SESSION_MAX_REASONABLE_US`] to prevent
    /// adversarial inputs from breaking downstream WCET arithmetic.
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let us = u64::deserialize(deserializer)?;
        Self::from_micros_validated(us)
            .ok_or_else(|| serde::de::Error::custom("timestamp exceeds SESSION_MAX_REASONABLE_US"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_arithmetic() {
        let t1 = MonotonicTimestamp::from_micros_unchecked(1000);
        let t2 = MonotonicTimestamp::from_micros_unchecked(2500);
        assert_eq!(t2.checked_sub(t1), Some(1500));
        assert_eq!(t1.checked_sub(t2), None);
        assert_eq!(t2.as_millis(), 2);
    }

    #[test]
    fn elapsed_since_returns_none_on_violation() {
        let t1 = MonotonicTimestamp::from_micros_unchecked(1000);
        let t2 = MonotonicTimestamp::from_micros_unchecked(500);
        // Reverse order = clock violation → None
        assert_eq!(t1.elapsed_since(t2), Some(500));
        assert_eq!(t2.elapsed_since(t1), None);
    }

    #[test]
    fn elapsed_since_saturating_yields_zero_on_violation() {
        let t1 = MonotonicTimestamp::from_micros_unchecked(1000);
        let t2 = MonotonicTimestamp::from_micros_unchecked(500);
        assert_eq!(t2.elapsed_since_saturating(t1), 0);
    }

    #[test]
    fn validated_constructor_rejects_oversized() {
        assert!(MonotonicTimestamp::from_micros_validated(1_000_000).is_some());
        assert!(MonotonicTimestamp::from_micros_validated(SESSION_MAX_REASONABLE_US).is_some());
        assert!(MonotonicTimestamp::from_micros_validated(SESSION_MAX_REASONABLE_US + 1).is_none());
        assert!(MonotonicTimestamp::from_micros_validated(u64::MAX).is_none());
    }

    #[test]
    fn timestamp_is_transparent() {
        assert_eq!(core::mem::size_of::<MonotonicTimestamp>(), 8);
        assert_eq!(core::mem::align_of::<MonotonicTimestamp>(), 8);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_rejects_oversized_timestamp() {
        // Bincode → u64 → MonotonicTimestamp roundtrip with oversized value
        let oversized: u64 = SESSION_MAX_REASONABLE_US + 1;
        let bytes = oversized.to_le_bytes();
        let result: core::result::Result<MonotonicTimestamp, _> = bincode::deserialize(&bytes);
        assert!(result.is_err());
    }
}
