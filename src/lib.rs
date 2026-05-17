// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org
//
// Security-audit-hardened BCI SDK
//   - compile_error! on unimplemented security paths (no runtime surprises)
//   - Mutex poison recovery (no panic in sync primitives)
//   - Explicit stub naming (MeshClientStub)
//   - MonotonicTimestamp with WCET documentation
//   - Fixed-point Q0.16 format explicitly documented
//   - deny(unsafe_code) with audited unsafe module for future zero-copy IPC

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
// Documentation lints are downgraded to allow for now; will tighten
// when full doc coverage is verified. `deny(unsafe_code)` is kept.
#![allow(missing_docs)]
#![allow(missing_debug_implementations)]
#![allow(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # AxonOS SDK
//!
//! Hardened SDK for brain-computer interface applications on the AxonOS
//! cognitive operating system.
//!
//! ## Security model
//!
//! - `no_std` by default — suitable for Cortex-M4F/M33 Secure World.
//! - Fixed-point wire format — deterministic across x86_64 and ARM.
//! - Capability-based access control — kernel-enforced, not client-honored.
//! - HMAC-SHA256 truncated attestation — every observation cryptographically
//!   bound to session key.
//!
//! ## Fixed-point format
//!
//! Confidence/score values use **Q0.16 unsigned fixed-point**:
//! - `0` = 0.0
//! - `65535` (`u16::MAX`) = 1.0
//! - Scaling factor: `value / 65535.0`
//!
//! This eliminates cross-architecture floating-point non-determinism.
//!
//! ## Time model
//!
//! All timestamps are [`MonotonicTimestamp`] — microseconds since session
//! start, guaranteed monotonic within a session. No wall-clock time is
//! exposed to applications (privacy boundary).
//!
//! ## Feature flags
//!
//! | Feature | Purpose |
//! |:---|:---|
//! | `std` | Hosted builds with IPC transport |
//! | `alloc` | Heap allocation without `std` |
//! | `serde` | JSON/CBOR wire serialization |
//! | `zerocopy` | Zero-copy FFI helpers |
//! | `kernel-stub` | **Development only.** Allows compilation without kernel ABI. |
//!
//! **⚠️ `kernel-stub` must NEVER be enabled in production.** It disables
//! cryptographic attestation verification and replaces it with no-ops.

pub mod capability;
pub mod error;
pub mod intent;
pub mod manifest;
pub mod mesh;
pub mod stream;
pub mod time;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
pub mod host;

// Audited unsafe module for future zero-copy IPC. Currently empty.
// All unsafe code in this crate MUST live in this module.
#[cfg(feature = "zerocopy")]
#[allow(unsafe_code)]
pub(crate) mod zerocopy_ext;

pub use capability::{Capability, CapabilitySet, RawCapabilitySet};
pub use error::{Error, Result};
pub use intent::{
    Direction, IntentKind, IntentObservation, Load, Quality,
};
pub use manifest::{Manifest, ManifestBuilder};
pub use mesh::{ConsentScope, MeshClientStub, WithdrawReason};
pub use stream::{IntentStream, ObservationFilter, OverflowPolicy, Subscription};
pub use time::MonotonicTimestamp;

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AxonOS Consent Protocol version.
pub const CONSENT_PROTOCOL_VERSION: &str = "0.2.0";

/// Kernel ABI version.
pub const KERNEL_ABI_VERSION: u32 = 1;

/// Q0.16 fixed-point denominator. `confidence_raw / CONFIDENCE_DENOM` = [0.0, 1.0].
pub const CONFIDENCE_DENOM: u16 = u16::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!CONSENT_PROTOCOL_VERSION.is_empty());
        assert!(KERNEL_ABI_VERSION >= 1);
    }

    #[test]
    fn confidence_denom_is_u16_max() {
        assert_eq!(CONFIDENCE_DENOM, 65535);
    }
}
