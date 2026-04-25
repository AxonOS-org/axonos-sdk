# Changelog

All notable changes to `axonos-sdk` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] — 2026-04-25

Maintenance release. No public API changes; downstream code does not need
modification (apart from bumping the minimum Rust toolchain).

### Fixed

- **Build with `--features serde` no longer fails.** `Capability`, `CapabilitySet`, and `PeerId` now derive `Serialize`/`Deserialize` under the `serde` feature, and the feature now activates `heapless/serde` and `siphasher/serde` so transitive bounds resolve.
- **Build with `RUSTFLAGS=-D warnings` is now clean.** Removed unused `Error` import in `stream.rs` (was only referenced from doc comments). Gated `core::fmt` import in `error.rs` under the `#[cfg(not(feature = "std"))]` block where it is actually used.
- **`FrameTooLarge` error variant** now documents its `size` and `max` fields.
- **Removed dead `dep_alloc` feature** that was a placeholder activating nothing. The `std` feature now directly implies `alloc`.
- **Removed dead `extern crate alloc;`** — no source file referenced any `alloc::*` types, so the declaration was unused.

### Changed

- **MSRV bumped from 1.75 to 1.85.** Required because the modern `proptest` and `criterion` dev-dependencies pull in transitive crates that require `edition2024` (stabilised in Rust 1.85). Library users on stable Rust are unaffected; only contributors running the test suite need 1.85+.
- Cargo profile and feature-flag comments expanded for clarity. Profile rationale (especially `debug = true` on release for embedded probe-rs symbols) now references RFC-0003.

### Maintenance

- All five feature combinations (`default`, `std`, `alloc`, `serde`, `zerocopy`, and their union) verified to build cleanly under `RUSTFLAGS=-D warnings` on the local toolchain matching the CI configuration.

## [0.1.0] — 2026-04-19

Initial public release.

### Added

- Core types: `IntentObservation` (32-byte, `Copy`, `#[repr(C)]`), `IntentKind`, `Direction`, `Load`, `Quality`.
- Capability model: `Capability` enum, `CapabilitySet` bitfield, per-capability kernel rate limits.
- Manifest builder with local validation (`app_id` length, rate vs kernel limits).
- Stream API: `IntentStream`, `StreamConfig`, `ObservationFilter`, `OverflowPolicy`, `Subscription`.
- Mesh integration: `MeshClient`, `ConsentScope`, `PeerId`, `WithdrawReason` — facade over the MMP Consent Extension v0.1.0.
- Error taxonomy: four-layer (L1 transport / L2 capability / L3 consent / L4 protocol) with stable `ErrorCode` wire codes and `is_terminal()` classifier.
- Host integration (`std` feature): `connect_local`, `InMemoryFixture` test harness, `AXONOS_ENDPOINT` env override.
- Five runnable examples: `hello_intent`, `mind_cursor`, `focus_monitor`, `mesh_coupling`, `bare_metal_no_std`.
- Integration test suite + Criterion benchmarks.
- `#![forbid(unsafe_code)]` at the crate root.
- `#![warn(missing_docs)]` and `#![warn(clippy::pedantic)]` CI-enforced.
- No-std build verified on `thumbv7em-none-eabihf` and `thumbv8m.main-none-eabihf`.
- Compile-time assertion: `IntentObservation` is exactly 32 bytes.
- Dual license: Apache-2.0 OR MIT.

### Compatibility

- MMP Consent Extension version targeted: **0.1.0**.
- AxonOS kernel ABI version: **1**.
- Minimum supported Rust version (MSRV): **1.85.0**.

[Unreleased]: https://github.com/AxonOS-org/axonos-sdk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.0
