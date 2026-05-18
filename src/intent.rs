// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Intent observation types — application-facing data model.
//!
//! # Fixed-point confidence (Q0.16)
//!
//! Confidence is represented as **unsigned Q0.16 fixed-point**:
//! ```text
//! value_f32 = confidence_raw / 65535.0
//! ```
//!
//! | Raw (`u16`) | Float equivalent | Meaning |
//! |:---|:---|:---|
//! | 0 | 0.0 | Zero confidence |
//! | 32768 | ~0.500 | Medium confidence |
//! | 58982 | ~0.900 | High confidence |
//! | 65535 | 1.0 | Full confidence |
//!
//! This format is deterministic across all architectures — x86_64 SSE,
//! Cortex-M4F FPU, and soft-float targets produce identical raw values.
//!
//! # Portable layout
//!
//! `IntentObservation` is `#[repr(C, align(8))]` — 32 bytes on both
//! 64-bit hosts and 32-bit embedded targets.

#![allow(clippy::cast_possible_truncation)]

use crate::time::MonotonicTimestamp;
use core::fmt;

/// A single intent observation. Always 32 bytes, always `Copy`.
///
/// # Layout (stable across `KERNEL_ABI_VERSION == 1`)
///
/// | Offset | Size | Field |
/// |:---|:---|:---|
/// | 0 | 8 | `timestamp_us` — [`MonotonicTimestamp`] |
/// | 8 | 2 | `kind_tag` |
/// | 10 | 2 | `quality_raw` — Q0.16 confidence |
/// | 12 | 4 | `payload` |
/// | 16 | 8 | `session_id` |
/// | 24 | 8 | `attestation` — truncated HMAC-SHA256 |
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct IntentObservation {
    pub(crate) timestamp_us: u64,
    pub(crate) kind_tag: u16,
    pub(crate) quality_raw: u16,
    pub(crate) payload: [u8; 4],
    pub(crate) session_id: u64,
    pub(crate) attestation: [u8; 8],
}

// Compile-time layout assertions — target-gated for portability.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<IntentObservation>() == 32);
const _: () = assert!(core::mem::align_of::<IntentObservation>() == 8);

impl IntentObservation {
    /// Construct a Direction observation.
    ///
    /// `confidence` is Q0.16 fixed-point: `65535 == 1.0`.
    /// Use [`crate::CONFIDENCE_DENOM`] for conversions.
    #[must_use]
    pub fn new_direction(
        timestamp: MonotonicTimestamp,
        dir: Direction,
        confidence: u16,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = dir as u8;
        Self {
            timestamp_us: timestamp.as_micros(),
            kind_tag: KindTag::DIRECTION,
            quality_raw: confidence,
            payload,
            session_id,
            attestation,
        }
    }

    /// Construct a Load observation.
    #[must_use]
    pub fn new_load(
        timestamp: MonotonicTimestamp,
        load: Load,
        confidence: u16,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = load as u8;
        Self {
            timestamp_us: timestamp.as_micros(),
            kind_tag: KindTag::LOAD,
            quality_raw: confidence,
            payload,
            session_id,
            attestation,
        }
    }

    /// Construct a Quality observation. Confidence is always `u16::MAX`.
    #[must_use]
    pub fn new_quality(
        timestamp: MonotonicTimestamp,
        quality: Quality,
        session_id: u64,
        attestation: [u8; 8],
    ) -> Self {
        let mut payload = [0u8; 4];
        payload[0] = quality as u8;
        Self {
            timestamp_us: timestamp.as_micros(),
            kind_tag: KindTag::QUALITY,
            quality_raw: u16::MAX,
            payload,
            session_id,
            attestation,
        }
    }

    /// Timestamp as [`MonotonicTimestamp`].
    #[must_use]
    pub const fn timestamp(&self) -> MonotonicTimestamp {
        MonotonicTimestamp::from_micros_unchecked(self.timestamp_us)
    }

    /// Raw timestamp in microseconds.
    #[must_use]
    pub const fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    /// Q0.16 fixed-point confidence. `u16::MAX == 1.0`.
    #[must_use]
    pub const fn confidence_raw(&self) -> u16 {
        self.quality_raw
    }

    /// Confidence as f32 for **display only**. Do not use for comparison
    /// or decision logic — use `confidence_raw()` instead.
    ///
    /// # Non-determinism warning
    /// This conversion uses floating-point division and may produce
    /// slightly different results across architectures. Always compare
    /// raw values for correctness.
    #[must_use]
    pub fn confidence_f32(&self) -> f32 {
        f32::from(self.quality_raw) / f32::from(crate::CONFIDENCE_DENOM)
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

    /// Decoded intent kind. Returns `Unknown` for unrecognized tags.
    #[must_use]
    pub fn kind(&self) -> IntentKind {
        match self.kind_tag {
            KindTag::DIRECTION => Direction::from_u8(self.payload[0])
                .map_or(IntentKind::Unknown, IntentKind::Direction),
            KindTag::LOAD => {
                Load::from_u8(self.payload[0]).map_or(IntentKind::Unknown, IntentKind::Load)
            }
            KindTag::QUALITY => {
                Quality::from_u8(self.payload[0]).map_or(IntentKind::Unknown, IntentKind::Quality)
            }
            _ => IntentKind::Unknown,
        }
    }
}

impl fmt::Debug for IntentObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntentObservation")
            .field("timestamp", &self.timestamp())
            .field("kind", &self.kind())
            .field("confidence_raw", &self.quality_raw)
            .field("session_id", &format_args!("{:#018x}", self.session_id))
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IntentObservation {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("IntentObservation", 5)?;
        st.serialize_field("timestamp_us", &self.timestamp_us)?;
        st.serialize_field("kind", &self.kind())?;
        st.serialize_field("confidence_raw", &self.quality_raw)?;
        st.serialize_field("session_id", &self.session_id)?;
        st.serialize_field("attestation", &self.attestation)?;
        st.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IntentObservation {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::fmt;
        use serde::de::{self, MapAccess, Visitor};

        struct ObsVisitor;

        impl<'de> Visitor<'de> for ObsVisitor {
            type Value = IntentObservation;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct IntentObservation")
            }

            fn visit_map<V>(self, mut map: V) -> Result<IntentObservation, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut timestamp_us = None;
                let mut kind = None::<IntentKind>;
                let mut confidence_raw = None::<u16>;
                let mut session_id = None;
                let mut attestation = None::<[u8; 8]>;

                while let Some(key) = map.next_key::<&str>()? {
                    match key {
                        "timestamp_us" => timestamp_us = Some(map.next_value()?),
                        "kind" => kind = Some(map.next_value()?),
                        "confidence_raw" => confidence_raw = Some(map.next_value()?),
                        "session_id" => session_id = Some(map.next_value()?),
                        "attestation" => attestation = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let timestamp_us =
                    timestamp_us.ok_or_else(|| de::Error::missing_field("timestamp_us"))?;
                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let confidence_raw = confidence_raw.unwrap_or(u16::MAX);
                let session_id =
                    session_id.ok_or_else(|| de::Error::missing_field("session_id"))?;
                let attestation =
                    attestation.ok_or_else(|| de::Error::missing_field("attestation"))?;

                let obs = match kind {
                    IntentKind::Direction(d) => IntentObservation::new_direction(
                        MonotonicTimestamp::from_micros_unchecked(timestamp_us),
                        d,
                        confidence_raw,
                        session_id,
                        attestation,
                    ),
                    IntentKind::Load(l) => IntentObservation::new_load(
                        MonotonicTimestamp::from_micros_unchecked(timestamp_us),
                        l,
                        confidence_raw,
                        session_id,
                        attestation,
                    ),
                    IntentKind::Quality(q) => IntentObservation::new_quality(
                        MonotonicTimestamp::from_micros_unchecked(timestamp_us),
                        q,
                        session_id,
                        attestation,
                    ),
                    IntentKind::Unknown => {
                        return Err(de::Error::custom(
                            "cannot deserialize Unknown into concrete observation",
                        ));
                    }
                };
                Ok(obs)
            }
        }

        d.deserialize_struct(
            "IntentObservation",
            &[
                "timestamp_us",
                "kind",
                "confidence_raw",
                "session_id",
                "attestation",
            ],
            ObsVisitor,
        )
    }
}

/// Internal discriminant tags.
struct KindTag;
impl KindTag {
    const DIRECTION: u16 = 0x0001;
    const LOAD: u16 = 0x0002;
    const QUALITY: u16 = 0x0003;
}

/// Classified intent kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum IntentKind {
    Direction(Direction),
    Load(Load),
    Quality(Quality),
    Unknown,
}

/// Cardinal direction for navigation intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Load {
    Low = 0,
    Moderate = 1,
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

/// Signal quality class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[repr(u8)]
pub enum Quality {
    High = 0,
    Moderate = 1,
    Low = 2,
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
    fn observation_is_32_bytes_on_64bit() {
        assert_eq!(core::mem::size_of::<IntentObservation>(), 32);
    }

    #[test]
    fn observation_align_is_8() {
        assert_eq!(core::mem::align_of::<IntentObservation>(), 8);
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
            let ts = MonotonicTimestamp::from_micros_unchecked(0);
            let obs = IntentObservation::new_direction(ts, d, 32768, 0, [0u8; 8]);
            assert_eq!(obs.kind(), IntentKind::Direction(d));
        }
    }

    #[test]
    fn unknown_tag_maps_to_unknown() {
        let ts = MonotonicTimestamp::from_micros_unchecked(0);
        let mut obs = IntentObservation::new_direction(ts, Direction::Up, 0, 0, [0u8; 8]);
        obs.kind_tag = 0xFFFF;
        assert_eq!(obs.kind(), IntentKind::Unknown);
    }

    #[test]
    fn confidence_is_fixed_point() {
        let ts = MonotonicTimestamp::from_micros_unchecked(0);
        let obs = IntentObservation::new_direction(ts, Direction::Up, 65535, 0, [0u8; 8]);
        assert_eq!(obs.confidence_raw(), 65535);
        assert!((obs.confidence_f32() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn timestamp_is_monotonic_type() {
        let ts = MonotonicTimestamp::from_micros_unchecked(1234);
        let obs = IntentObservation::new_direction(ts, Direction::Up, 0, 0, [0u8; 8]);
        assert_eq!(obs.timestamp().as_micros(), 1234);
    }
}

