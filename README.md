# AxonOS SDK

[![Crates.io](https://img.shields.io/crates/v/axonos-sdk)](https://crates.io/crates/axonos-sdk)
[![Docs.rs](https://docs.rs/axonos-sdk/badge.svg)](https://docs.rs/axonos-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue.svg)](LICENSE-APACHE)
[![Rust Version](https://img.shields.io/badge/rustc-1.85+-orange.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![CI](https://github.com/AxonOS-org/axonos-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/AxonOS-org/axonos-sdk/actions/workflows/ci.yml)
[![Safety Critical](https://img.shields.io/badge/safety-critical-red.svg)](SECURITY.md)
[![no_std](https://img.shields.io/badge/no__std-supported-success.svg)](https://docs.rust-embedded.org/book/intro/no-std.html)

**Hardened SDK for the AxonOS cognitive operating system.**

Typed intent events, capability manifests, and MMP consent integration for brain-computer interface applications.

> **Version 0.3.0** — production-hardened pre-release. See [readiness checklist](#pre-release-readiness).

---

## What this is

`axonos-sdk` is the **public contract** between a BCI application and the AxonOS kernel. Applications receive typed, cryptographically attested **intent observations** — not raw neural signals. Raw EEG never crosses the partition.

## What this isn't

- Not a signal-processing library. Classifier and artifact rejection live in the kernel.
- Not a medical device. A shippable BCI requires a certified kernel, qualified toolchain, and full IEC 62304 lifecycle documentation.
- Not a direct interface to Neuralink, Synchron, or any other specific BCI device.

## Install

```toml
[dependencies]
axonos-sdk = "0.3"
```

Hosted builds:
```toml
[dependencies]
axonos-sdk = { version = "0.3", features = ["std", "serde"] }
```

Bare-metal Cortex-M:
```toml
[dependencies]
axonos-sdk = { version = "0.3", default-features = false }
```

## Feature flags

| Feature | Purpose |
|:---|:---|
| `std` | Hosted builds with IPC transport |
| `alloc` | Heap allocation without `std` |
| `serde` | JSON/CBOR serialization |
| `zerocopy` | Zero-copy FFI helpers |
| `kernel-stub` | **Development only.** Allows compilation without kernel ABI. **Never in production.** |

## Quickstart

```rust
use axonos_sdk::{Capability, Direction, IntentKind, IntentStream, Manifest, MonotonicTimestamp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::builder()
        .app_id("com.example.cursor")
        .capability(Capability::Navigation)
        .max_rate_hz(50)
        .build()?;

    let mut stream = IntentStream::connect(&manifest)?;

    while let Some(obs) = stream.try_next()? {
        if let IntentKind::Direction(d) = obs.kind() {
            println!("cursor: {:?} (confidence {:.0}%)", d, obs.confidence_f32() * 100.0);
        }
    }
    Ok(())
}
```

## Fixed-point confidence (Q0.16)

Confidence uses **unsigned Q0.16 fixed-point** for cross-architecture determinism:

| Raw (`u16`) | Float | Meaning |
|:---|:---|:---|
| 0 | 0.0 | Zero confidence |
| 32768 | ~0.500 | Medium |
| 58982 | ~0.900 | High |
| 65535 | 1.0 | Full confidence |

```rust
let confidence: u16 = 58982; // ~90%
let float = confidence as f32 / 65535.0; // display only
```

## Time model

All timestamps are [`MonotonicTimestamp`] — microseconds since session start, guaranteed monotonic. No wall-clock time is exposed (privacy boundary).

| Operation | WCET (Cortex-M4F @ 168 MHz) |
|:---|:---|
| `as_micros()` | 1 cycle |
| `as_millis()` | ~10 cycles |
| `checked_sub()` | 3 cycles |

## Capability model

| Capability | Events | Kernel limit |
|:---|:---|:---|
| `Navigation` | Direction | 50 Hz |
| `WorkloadAdvisory` | Cognitive load | 1 Hz |
| `SessionQuality` | Signal quality | 2 Hz |
| `ArtifactEvents` | Artifacts | 10 Hz |

**Prohibited** (kernel-rejected): raw EEG, emotion inference, cognitive profile read, re-identification.

## Safety architecture

- `#![deny(unsafe_code)]` — all unsafe isolated in `zerocopy_ext` module (audited).
- **No runtime panic in sync primitives** — mutex poison recovery via `into_inner()`.
- **Compile-time security** — `try_next()` is `compile_error!` without `kernel-stub`.
- **Explicit stubs** — `MeshClientStub`, not fake `MeshClient`.

## Contributing

```sh
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo build --target thumbv7em-none-eabihf --no-default-features --features kernel-stub
```

Security: see [`SECURITY.md`](SECURITY.md) — **do not** open public issues.

## License

Dual-licensed under Apache-2.0 or MIT. Every source file carries an SPDX identifier.

---

## Pre-release readiness

**Shipped in 0.3.0:**
- `IntentObservation` (32-byte, `#[repr(C, align(8))]`), fixed-point Q0.16
- `MonotonicTimestamp` with WCET documentation
- `CapabilitySet` u32 bitfield + formal wire spec + audit formatting
- `ManifestBuilder` with infallible intermediates, no silent truncation
- `IntentStream` with `compile_error!` security boundary
- `MeshClientStub` — explicitly marked, not fake
- Mutex poison recovery (no panic)
- `#![deny(unsafe_code)]` with audited `zerocopy_ext` module
- 5 examples, integration tests, Criterion benchmarks

**Pending before 1.0:**
- Real IPC transport in `host.rs`
- HMAC-SHA256 attestation verification
- `axonos-consent` wire integration
- Kernel ABI stabilization

**Practical meaning:** types and API are production-quality. Runtime I/O is placeholder until AxonOS kernel ships. Build against this SDK today; run against real kernel when available.

---

axonos.org · [medium.com/@AxonOS](https://medium.com/@AxonOS) · info@axonos.org
