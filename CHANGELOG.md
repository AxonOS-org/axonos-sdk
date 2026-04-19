# Changelog

All notable changes to `axonos-sdk` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- Minimum supported Rust version (MSRV): **1.75.0**.

[Unreleased]: https://github.com/AxonOS-org/axonos-sdk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.0
