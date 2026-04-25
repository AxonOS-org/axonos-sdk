// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS
//
// This file is part of the axonos-sdk public SDK for the AxonOS cognitive
// operating system. It is dual-licensed under the Apache License, Version 2.0
// or the MIT License, at your option.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)] // documented at module level
#![allow(clippy::missing_panics_doc)] // no panics in public API
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # AxonOS SDK
//!
//! Public SDK for building applications on the AxonOS cognitive operating
//! system — a deterministic, real-time operating system for brain-computer
//! interfaces.
//!
//! ## Scope
//!
//! This crate is the **application-facing contract** between a BCI application
//! and the AxonOS kernel. Applications receive **typed intent events** with
//! cryptographic attestation; they do not receive raw neural signals. This
//! design choice is fundamental to the AxonOS privacy and safety model —
//! raw EEG never leaves the Secure World partition, and applications operate
//! on classified intent observations only.
//!
//! ## Conformance
//!
//! This SDK targets **general conformance** per MMP Consent Extension §10.1.
//! The safety-critical tier (§10.2 — NVRAM persistence, hardware stimulation
//! lockout, no auto-reconnect after power cycle) is implemented in the
//! `axonos-kernel` crate, which is not part of this SDK.
//!
//! ## Feature flags
//!
//! - `std` — enables `std`-dependent integrations (`std::error::Error` impls,
//!   `thiserror` derives, `Box<dyn Error>` in public APIs). **Required for
//!   most examples.**
//! - `alloc` — enables heap allocation without requiring `std`. Suitable for
//!   Cortex-M33 targets with a global allocator.
//! - `serde` — enables Serde serialization for intent events over JSON or CBOR.
//! - `zerocopy` — enables zero-copy deserialization helpers for FFI and ring
//!   buffer protocols.
//!
//! ## Quickstart
//!
//! ```no_run
//! # #[cfg(feature = "std")]
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use axonos_sdk::{IntentStream, IntentKind, Subscription};
//!
//! // Declare a capability manifest (what the app is allowed to observe).
//! let manifest = axonos_sdk::Manifest::builder()
//!     .app_id("com.example.cursor")
//!     .capability(axonos_sdk::Capability::Navigation)
//!     .max_rate_hz(50)
//!     .build()?;
//!
//! // Subscribe to intent events within the declared capabilities.
//! let mut stream = IntentStream::connect(&manifest)?;
//!
//! while let Some(event) = stream.next()? {
//!     match event.kind() {
//!         IntentKind::Direction(d) => println!("cursor: {:?}", d),
//!         _ => (),
//!     }
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "std"))]
//! # fn main() {}
//! ```
//!
//! ## See also
//!
//! - [`axonos-consent`](https://crates.io/crates/axonos-consent) — protocol-level
//!   consent primitive (MMP Consent Extension v0.1.0, IEC 62304 hygiene).
//! - [axonos.org](https://axonos.org) — project homepage.
//! - [Article #12 — Benchmark report](https://medium.com/@AxonOS/axonos-mvp-the-benchmark-report-latency-power-ea6c78d0e091)
//!   — canonical performance data.

// ─── Public module tree ──────────────────────────────────────────────────
pub mod capability;
pub mod error;
pub mod intent;
pub mod manifest;
pub mod mesh;
pub mod stream;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod host;

// ─── Prelude re-exports ──────────────────────────────────────────────────
pub use capability::{Capability, CapabilitySet};
pub use error::{Error, Result};
pub use intent::{
    Direction, IntentKind, IntentObservation, Load, Quality, Timestamp,
};
pub use manifest::{Manifest, ManifestBuilder};
pub use mesh::{ConsentScope, MeshClient};
pub use stream::{IntentStream, ObservationFilter, OverflowPolicy, Subscription};

/// Crate version string, matching `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocol compatibility — the MMP Consent Extension version this SDK targets.
pub const MMP_CONSENT_VERSION: &str = "0.1.0";

/// AxonOS kernel ABI version this SDK is compatible with.
pub const KERNEL_ABI_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constants_are_non_empty() {
        assert!(!VERSION.is_empty());
        assert!(!MMP_CONSENT_VERSION.is_empty());
        assert!(KERNEL_ABI_VERSION >= 1);
    }
}
