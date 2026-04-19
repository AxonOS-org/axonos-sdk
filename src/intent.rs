// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Intent observation types — the application-facing data model.
//!
//! # What applications see
//!
//! Applications do **not** receive raw neural signals. Instead, they receive
//! typed [`IntentObservation`] events that have already been classified,
//! validated, and cryptographically attested by the AxonOS signal processing
//! pipeline. This boundary is fundamental to the privacy and safety model:
//!
//! - **Privacy**: raw EEG never leaves the Secure World partition. An
//!   attacker who fully compromises an AxonOS application cannot extract
//!   the user's raw neural signals, because those signals are not on the
//!   application side of the partition.
//! - **Safety**: the classifier output is the smallest signal that is useful
//!   to the application. The application cannot reconstruct finer-grained
//!   mental state from an intent observation.
//!
//! # Event layout
//!
//! [`IntentObservation`] is **32 bytes**, `Copy`, `#[repr(C)]`, and suitable
//! for zero-copy transport over FFI, shared memory, or a ring buffer. Every
//! field has explicit bit-width and byte offset — this is a stable wire
//! format.

// All u8 casts in this module are from #[repr(u8)] enums and are safe.
#![allow(clippy::cast_possible_truncation)]

use core::fmt;

/// A single intent observation delivered from the AxonOS kernel to the
/// application.
///
/// Always 32 bytes. Always `Copy`. Layout is stable across kernel versions
/// with the same [`crate::KERNEL_ABI_VERSION`].
///
/// # Layout
///
/// | Offset | Size | Field |
/// |:---|:---:|:---|
/// | 0 | 8 | `timestamp_us` — microseconds since session start |
/// | 8 | 2 | `kind_tag` — discriminant for [`IntentKind`] |
/// | 10 | 2 | `quality` — classifier confidence (u16 / 65535.0) |
/// | 12 | 4 | `payload` — kind-specific payload (see `IntentKind`) |
/// | 16 | 8 | `session_id` — opaque session identifier |
/// | 24 | 8 | `attestation` — truncated HMAC-SHA256 tag (first 8 bytes) |
///
/// # Example
///
/// ```
/// use axonos_sdk::{IntentObservation, IntentKind, Direction};
///
/// let obs = IntentObservation::new_direction(
///     12_345,                        // timestamp_us
///     Direction::Up,
///     0.84,                          // confidence
///     0xDEAD_BEEF_u64,               // session_id
///     [0u8; 8],                      // attestation (in real code from kernel)
/// );
///
/// assert_eq!(obs.kind(), IntentKind::Direction(Direction::Up));
/// assert!((obs.confidence() - 0.84).abs() < 0.01);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IntentObservation {
    /// Monotonic microseconds since session start.
    pub(crate) timestamp_us: u64,
    /// Discriminant. See [`IntentKind`].
    pub(crate) kind_tag: u16,
    /// Quality / confidence score. `u16::MAX` == 1.0.
    pub(crate) quality_raw: u16,
    /// Payload — interpretation depends on `kind_tag`.
    pub(crate) payload: [u8; 4],
    /// Opaque session identifier.
    pub(crate) session_id: u64,
    /// Truncated HMAC tag (first 8 bytes of HMAC-SHA256).
    pub(crate) attestation: [u8; 8],
}

// Compile-time layout assertion.
const _: () = assert!(core::mem::size_of::<IntentObservation>() == 32);
const _: () = assert!(core::mem::align_of::<IntentObservation>() == 8);

impl IntentObservation {
    /// Construct a Direction observation.
    #[must_use]
    pub fn new_direction(
        timestamp_us: u64,
        dir: Direction,
        confidence: f32,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = dir as u8;
        Self {
            timestamp_us,
            kind_tag: KindTag::DIRECTION,
            quality_raw: confidence_to_raw(confidence),
            payload,
            session_id,
            attestation,
        }
    }

    /// Construct a Load observation (cognitive workload).
    #[must_use]
    pub fn new_load(
        timestamp_us: u64,
        load: Load,
        confidence: f32,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = load as u8;
        Self {
            timestamp_us,
            kind_tag: KindTag::LOAD,
            quality_raw: confidence_to_raw(confidence),
            payload,
            session_id,
            attestation,
        }
    }

    /// Construct a Quality observation (signal quality assessment).
    #[must_use]
    pub fn new_quality(
        timestamp_us: u64,
        quality: Quality,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = quality as u8;
        Self {
            timestamp_us,
            kind_tag: KindTag::QUALITY,
            quality_raw: u16::MAX, // quality events are always full-confidence
            payload,
            session_id,
            attestation,
        }
    }

    /// Timestamp in microseconds since session start.
    #[must_use]
    pub const fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    /// Timestamp wrapper type for richer APIs.
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        Timestamp(self.timestamp_us)
    }

    /// Classifier confidence in the range `[0.0, 1.0]`.
    #[must_use]
    pub fn confidence(&self) -> f32 {
        f32::from(self.quality_raw) / f32::from(u16::MAX)
    }

    /// Opaque session identifier.
    #[must_use]
    pub const fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Attestation tag (truncated HMAC-SHA256).
    #[must_use]
    pub const fn attestation(&self) -> &[u8; 8] {
        &self.attestation
    }

    /// Decoded intent kind. Returns `None` if the `kind_tag` is unrecognized —
    /// this occurs when the SDK is older than the kernel.
    #[must_use]
    pub fn kind(&self) -> IntentKind {
        match self.kind_tag {
            KindTag::DIRECTION => {
                if let Some(d) = Direction::from_u8(self.payload[0]) {
                    IntentKind::Direction(d)
                } else {
                    IntentKind::Unknown
                }
            }
            KindTag::LOAD => {
                if let Some(l) = Load::from_u8(self.payload[0]) {
                    IntentKind::Load(l)
                } else {
                    IntentKind::Unknown
                }
            }
            KindTag::QUALITY => {
                if let Some(q) = Quality::from_u8(self.payload[0]) {
                    IntentKind::Quality(q)
                } else {
                    IntentKind::Unknown
                }
            }
            _ => IntentKind::Unknown,
        }
    }
}

impl fmt::Debug for IntentObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntentObservation")
            .field("timestamp_us", &self.timestamp_us)
            .field("kind", &self.kind())
            .field("confidence", &self.confidence())
            .field("session_id", &format_args!("{:#018x}", self.session_id))
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IntentObservation {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("IntentObservation", 5)?;
        st.serialize_field("timestamp_us", &self.timestamp_us)?;
        st.serialize_field("kind", &self.kind())?;
        st.serialize_field("confidence", &self.confidence())?;
        st.serialize_field("session_id", &self.session_id)?;
        st.serialize_field("attestation", &self.attestation)?;
        st.end()
    }
}

fn confidence_to_raw(c: f32) -> u16 {
    let clamped = if c < 0.0 {
        0.0
    } else if c > 1.0 {
        1.0
    } else {
        c
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        (clamped * f32::from(u16::MAX)) as u16
    }
}

/// Timestamp wrapper, microseconds since session start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Raw microseconds value.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.0
    }

    /// Value in milliseconds, truncated.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0 / 1000
    }

    /// Duration since another timestamp, in microseconds.
    #[must_use]
    pub const fn checked_sub(self, earlier: Timestamp) -> Option<u64> {
        if self.0 >= earlier.0 {
            Some(self.0 - earlier.0)
        } else {
            None
        }
    }
}

/// Discriminant tags — internal to the wire format.
struct KindTag;
impl KindTag {
    const DIRECTION: u16 = 0x0001;
    const LOAD: u16 = 0x0002;
    const QUALITY: u16 = 0x0003;
}

/// Classified intent kind. Variants are discriminated at the wire level
/// by [`IntentObservation::kind_tag`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum IntentKind {
    /// A navigation direction intent.
    Direction(Direction),
    /// A cognitive workload assessment.
    Load(Load),
    /// A signal-quality event.
    Quality(Quality),
    /// The kernel delivered an event of a kind this SDK does not understand.
    /// Applications should ignore these gracefully (forward compatibility).
    Unknown,
}

/// Cardinal direction for navigation intents.
///
/// Values are deliberately chosen so the numeric order matches a clock-face
/// reading (Up=0, Right=1, Down=2, Left=3), which simplifies integration
/// with 2D cursor control loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Direction {
    /// Up.
    Up = 0,
    /// Right.
    Right = 1,
    /// Down.
    Down = 2,
    /// Left.
    Left = 3,
    /// Explicit no-direction / neutral.
    Neutral = 4,
}

impl Direction {
    const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Up),
            1 => Some(Self::Right),
            2 => Some(Self::Down),
            3 => Some(Self::Left),
            4 => Some(Self::Neutral),
            _ => None,
        }
    }
}

/// Cognitive workload class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Load {
    /// Low cognitive load.
    Low = 0,
    /// Moderate cognitive load.
    Moderate = 1,
    /// High cognitive load.
    High = 2,
}

impl Load {
    const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Low),
            1 => Some(Self::Moderate),
            2 => Some(Self::High),
            _ => None,
        }
    }
}

/// Signal quality class — reported periodically so applications can gracefully
/// degrade when electrode contact is poor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Quality {
    /// High-quality signal. Classifier output should be trusted.
    High = 0,
    /// Moderate quality. Applications should cross-check with other inputs.
    Moderate = 1,
    /// Low quality. Classifier output is unreliable and should typically be
    /// rejected by the application.
    Low = 2,
    /// No signal / electrodes detached.
    NoSignal = 3,
}

impl Quality {
    const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::High),
            1 => Some(Self::Moderate),
            2 => Some(Self::Low),
            3 => Some(Self::NoSignal),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_is_32_bytes() {
        assert_eq!(core::mem::size_of::<IntentObservation>(), 32);
    }

    #[test]
    fn direction_round_trip() {
        for d in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Neutral,
        ] {
            let obs = IntentObservation::new_direction(0, d, 0.5, 0, [0u8; 8]);
            assert_eq!(obs.kind(), IntentKind::Direction(d));
        }
    }

    #[test]
    fn unknown_tag_maps_to_unknown() {
        let mut obs = IntentObservation::new_direction(0, Direction::Up, 0.5, 0, [0u8; 8]);
        obs.kind_tag = 0xFFFF; // pretend kernel sent unknown type
        assert_eq!(obs.kind(), IntentKind::Unknown);
    }

    #[test]
    fn confidence_is_clamped() {
        let obs = IntentObservation::new_direction(0, Direction::Up, 2.0, 0, [0u8; 8]);
        assert!((obs.confidence() - 1.0).abs() < 0.001);
        let obs = IntentObservation::new_direction(0, Direction::Up, -0.5, 0, [0u8; 8]);
        assert!(obs.confidence().abs() < 0.001);
    }

    #[test]
    fn timestamp_arithmetic() {
        let t1 = Timestamp(1000);
        let t2 = Timestamp(2500);
        assert_eq!(t2.checked_sub(t1), Some(1500));
        assert_eq!(t1.checked_sub(t2), None);
        assert_eq!(t2.as_millis(), 2);
    }
}
