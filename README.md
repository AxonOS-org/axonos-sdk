<!--
SPDX-License-Identifier: Apache-2.0 OR MIT
Copyright (c) 2026 Denis Yermakou / AxonOS
-->

<h1 align="center">axonos-sdk</h1>

<p align="center">
  <strong>The application-facing SDK for the AxonOS cognitive operating system.</strong><br>
  Typed intent events, capability manifests, and MMP consent integration for brain-computer interface applications.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/status-pre--release-orange" alt="Status: pre-release">
  <img src="https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blueviolet" alt="License: Apache-2.0 OR MIT">
  <img src="https://img.shields.io/badge/MSRV-1.85-orange" alt="MSRV 1.85">
  <img src="https://img.shields.io/badge/unsafe-forbidden-success" alt="forbid(unsafe_code)">
  <img src="https://img.shields.io/badge/no__std-supported-success" alt="no_std supported">
</p>

> **Status: pre-release (0.1.0).** This crate has not yet been published to crates.io. API is considered stable in practice but reserves the right to minor adjustments before 1.0. See the bottom of this file for the honest readiness checklist.

---

## What this is

`axonos-sdk` is the **public contract** between a brain-computer interface application and the [AxonOS](https://axonos.org) kernel. Applications receive typed, cryptographically attested **intent observations** — not raw neural signals. This boundary is fundamental: raw EEG never crosses the partition to the application side.

If you are building on AxonOS, this is the crate you add to `Cargo.toml` once it is published.

## What this isn't

- Not a signal-processing library. The classifier, spatial filters, and artifact rejection live in the AxonOS kernel and are not part of this SDK.
- Not a medical device. This SDK is software tooling; a shippable BCI requires a certified kernel, qualified toolchain, and full IEC 62304 lifecycle documentation.
- Not a direct interface to Neuralink, Synchron, or any other specific BCI device. AxonOS defines its own open reference hardware and an open protocol stack.

## Install (once published)

```toml
[dependencies]
axonos-sdk = "0.1"
```

For hosted (std) builds with full I/O:

```toml
[dependencies]
axonos-sdk = { version = "0.1", features = ["std", "serde"] }
```

For bare-metal Cortex-M:

```toml
[dependencies]
axonos-sdk = { version = "0.1", default-features = false }
```

## Feature flags

| Feature | Default | What it enables |
|:---|:---:|:---|
| `std` | — | `std::error::Error` impls, `thiserror`, local IPC connection, `InMemoryFixture` |
| `alloc` | — | Heap allocation without `std` — for Cortex-M33 with a global allocator |
| `serde` | — | Serde serialization for intent events (JSON / CBOR wire formats) |
| `zerocopy` | — | Zero-copy deserialization helpers for FFI and ring buffers |

The default build is `no_std` with no heap allocation — suitable for the STM32F407/STM32H573 application partition.

## Quickstart

```rust
use axonos_sdk::{Capability, Direction, IntentKind, IntentStream, Manifest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Declare what the app is allowed to observe.
    let manifest = Manifest::builder()
        .app_id("com.example.cursor")?
        .capability(Capability::Navigation)
        .max_rate_hz(50)
        .build()?;

    // Connect to the local kernel.
    let mut stream = IntentStream::connect(&manifest)?;

    while let Some(obs) = stream.try_next()? {
        if let IntentKind::Direction(d) = obs.kind() {
            println!("cursor: {:?} (confidence {:.0}%)", d, obs.confidence() * 100.0);
        }
    }
    Ok(())
}
```

Run the examples:

```sh
cargo run --example hello_intent --features std
cargo run --example mind_cursor --features "std serde"
cargo run --example focus_monitor --features std
cargo run --example mesh_coupling --features "std serde"
cargo build --example bare_metal_no_std --no-default-features
```

## Capability model

An AxonOS application declares what it is **authorized to observe** in its `Manifest`. The kernel will reject manifests that request capabilities outside the public set below — there is no escape hatch for "give me raw EEG," by design.

| Capability | Event class | Kernel rate limit |
|:---|:---|:---:|
| `Navigation` | Direction events for cursor/menu control | 50 Hz |
| `WorkloadAdvisory` | Cognitive load (low/moderate/high) | 1 Hz |
| `SessionQuality` | Signal-quality indicator | 2 Hz |
| `ArtifactEvents` | Electrode / artifact notifications | 10 Hz |

**Explicitly prohibited** (kernel-rejected): raw EEG, continuous emotion inference, cognitive profile read, re-identification. These align with the UNESCO 2025 Recommendation on the Ethics of Neurotechnology §III.

## Privacy guarantees

The SDK encodes, at the type level, what an AxonOS application can and cannot see:

- **No raw signal APIs.** There is no function in this crate that returns EEG samples. There cannot be, because the kernel never sends them across the partition.
- **Rate limits are structural.** `Capability::kernel_rate_limit_hz()` returns the maximum event rate the kernel policy will deliver. Applications that declare a higher rate are rejected at handshake.
- **Observations are attested.** Every `IntentObservation` carries a truncated HMAC-SHA256 tag. Unattested events are rejected at the SDK boundary with `Error::AttestationFailed` (terminal).
- **Withdrawal is terminal.** When the user withdraws consent via the `MeshClient` or a hardware button, the stream returns `Error::ConsentWithdrawn` and will not resume without a fresh handshake. This follows MMP Consent Extension v0.1.0 §4.1.

## Integration with the MMP Consent Extension

`axonos-sdk::mesh::MeshClient` provides a typed facade for the four core consent operations:

| SDK call | MMP frame | Spec section |
|:---|:---|:---:|
| `withdraw_consent(Peer(x), reason)` | `consent-withdraw` scope=peer | §3.1 |
| `withdraw_consent(All, reason)` | `consent-withdraw` scope=all | §3.1 |
| `suspend_consent(scope)` | `consent-suspend` | §3.2 |
| `resume_consent(scope)` | `consent-resume` | §3.3 |

The actual wire implementation lives in [axonos-consent](https://github.com/AxonOS-org/axonos-consent) — `#![no_std]`, zero-allocation, 15/15 interop vectors passing against an independent Node.js implementation.

## Error taxonomy

All fallible operations return `Result<T, Error>`. Errors are layered:

- **L1 — transport**: `TransportUnreachable`, `AbiMismatch`
- **L2 — capability/quota**: `CapabilityNotDeclared`, `ManifestRejected`, `RateLimitExceeded`
- **L3 — consent**: `ConsentSuspended` (non-terminal), `ConsentWithdrawn` (terminal)
- **L4 — protocol**: `Protocol(ProtocolFault)`, `AttestationFailed`, `StreamOverflow`

Use `Error::is_terminal()` to decide whether to tear down the subscription or retry.

## MSRV

**Rust 1.85.0** (February 20, 2025).

## Safety and correctness

- `#![forbid(unsafe_code)]` at the crate root — **no `unsafe` blocks anywhere in this SDK**.
- `#![warn(missing_docs)]` — every public item is documented.
- `#![warn(clippy::pedantic)]` — pedantic lints enabled.
- Compile-time layout assertion: `IntentObservation` is **exactly 32 bytes**.
- Unit tests for every public type; integration tests via `InMemoryFixture`.
- Criterion benchmarks on the hot path.

## Enterprise support

A commercial support tier is available for teams building production BCI systems on AxonOS. See [`ENTERPRISE.md`](./ENTERPRISE.md) for details.

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Short version: pull requests welcome; please run `cargo test --all-features && cargo clippy --all-features -- -D warnings && cargo fmt --check` before opening.

Security issues: see [`SECURITY.md`](./SECURITY.md) — **do not** open public issues for security reports.

## License

Dual-licensed under [Apache-2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT) at your option. Every source file carries an SPDX identifier.

## Related projects

- [axonos-consent](https://github.com/AxonOS-org/axonos-consent) — MMP Consent Extension reference implementation.
- [AxonOS homepage](https://axonos.org)
- [AxonOS research series](https://medium.com/@AxonOS) — 39 articles documenting the architecture.
- [SVAF paper (arXiv:2604.03955)](https://arxiv.org/abs/2604.03955) — SYM.BOT's coupling engine, the context in which consent sits.

---

## Pre-release readiness — honest checklist

This section exists so that anyone evaluating the crate knows exactly what state it is in. It will be deleted from the README at the 1.0 release.

**Shipped in this repo:**

- [x] Core types: `IntentObservation` (32-byte, `Copy`, `#[repr(C)]`), `IntentKind`, `Direction`, `Load`, `Quality`
- [x] Capability model with per-capability kernel rate limits
- [x] `Manifest` + `ManifestBuilder` with local validation
- [x] `IntentStream` with `StreamConfig`, `ObservationFilter`, `OverflowPolicy`
- [x] `MeshClient` facade for MMP Consent Extension operations
- [x] Full error taxonomy with `ErrorCode` wire codes
- [x] `#![forbid(unsafe_code)]` at the crate root
- [x] Dual-license file structure (Apache-2.0 / MIT)
- [x] 5 example programs
- [x] Integration test skeleton
- [x] Criterion benchmark skeleton
- [x] CI workflow template (`.github/workflows/ci.yml`)

**Pending before 0.1.0 publish to crates.io:**

- [ ] `cargo build --all-features` green on stable Rust
- [ ] `cargo build --no-default-features` green for the `no_std` path
- [ ] `cargo test --all-features` green run
- [ ] `cargo clippy --all-features -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Build verification on `thumbv7em-none-eabihf` and `thumbv8m.main-none-eabihf` targets
- [ ] `cargo publish --dry-run` succeeds
- [ ] Initial CI run green on GitHub Actions

**Pending before 1.0:**

- [ ] Real IPC transport in `host.rs` (currently scripted observations only; wire transport to kernel is a stub)
- [ ] Real `try_next()` implementation (currently returns `None`; placeholder until kernel endpoint exists)
- [ ] Published performance numbers from Criterion runs
- [ ] First downstream user feedback incorporated
- [ ] Kernel ABI stabilization (coordinated with `axonos-kernel`)

**What this means practically:** the types and API surface are production-quality. The runtime I/O pieces (`connect_local`, `try_next`) are **placeholders** until the AxonOS kernel ships. You can build against this SDK today and your code will compile and run against the `InMemoryFixture`; you cannot yet run against a real AxonOS kernel because one has not been released.

---

<p align="center"><sub>
  <a href="https://axonos.org">axonos.org</a> ·
  <a href="https://medium.com/@AxonOS">medium.com/@AxonOS</a> ·
  <a href="mailto:axonosorg@gmail.com">axonosorg@gmail.com</a>
</sub></p>
