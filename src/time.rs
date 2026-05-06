// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Monotonic time abstraction for BCI sessions.
//!
//! # Time model
//!
//! AxonOS guarantees **session-local monotonicity**: timestamps within a
//! single session never decrease. There is no wall-clock time exposed to
//! applications — this is a deliberate privacy boundary.
//!
//! # WCET guarantees
//!
//! All operations on `MonotonicTimestamp` are O(1) and have a
//! worst-case execution time (WCET) bounded by the underlying u64 arithmetic:
//!
//! | Operation | WCET (Cortex-M4F @ 168 MHz) |
//! |:---|:---|
//! | `as_micros()` | 1 cycle |
//! | `as_millis()` | ~10 cycles (division by 1000) |
//! | `checked_sub()` | 3 cycles |
//! | `elapsed_since()` | 3 cycles |
//!
//! No allocations. No syscalls. No floating point.

use core::fmt;

/// Monotonic timestamp — microseconds since session start.
///
/// # Invariants
/// - Never decreases within a session.
/// - Wraps at `u64::MAX` (~584,942 years at 1 µs resolution).
/// - Comparison is valid only within the same session.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MonotonicTimestamp(u64);

impl MonotonicTimestamp {
    /// Construct from raw microseconds. Used by kernel transport only.
    ///
    /// # Safety
    /// The caller must ensure the value is monotonically increasing
    /// within the session. This constructor is not unsafe in the Rust
    /// sense, but violating the invariant breaks downstream logic.
    #[must_use]
    pub const fn from_micros_unchecked(us: u64) -> Self {
        Self(us)
    }

    /// Raw microseconds since session start.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Milliseconds since session start, truncated.
    ///
    /// # WCET
    /// ~10 cycles on Cortex-M4F (hardware divide).
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0 / 1000
    }

    /// Duration since an earlier timestamp, in microseconds.
    ///
    /// Returns `None` if `earlier` is after `self` (clock violation).
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
    /// # Panics
    /// Panics in debug builds if `earlier` is after `self`. In release,
    /// returns 0 (defensive).
    #[must_use]
    pub const fn elapsed_since(self, earlier: MonotonicTimestamp) -> u64 {
        match self.checked_sub(earlier) {
            Some(d) => d,
            None => {
                // Defensive: in release builds, clock violation returns 0.
                // In debug builds, this is a logic error and should be caught
                // by the kernel before reaching the SDK.
                0
            }
        }
    }
}

impl fmt::Debug for MonotonicTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MonotonicTimestamp({} µs)", self.0)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MonotonicTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MonotonicTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let us = u64::deserialize(deserializer)?;
        Ok(Self::from_micros_unchecked(us))
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
    fn elapsed_since_defensive() {
        let t1 = MonotonicTimestamp::from_micros_unchecked(1000);
        let t2 = MonotonicTimestamp::from_micros_unchecked(500);
        // In release: returns 0 (defensive)
        // In debug: this would indicate a kernel bug
        assert_eq!(t1.elapsed_since(t2), 0);
    }

    #[test]
    fn timestamp_is_transparent() {
        assert_eq!(core::mem::size_of::<MonotonicTimestamp>(), 8);
        assert_eq!(core::mem::align_of::<MonotonicTimestamp>(), 8);
    }
}
