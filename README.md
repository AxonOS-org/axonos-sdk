<div align="center">

# axonos-sdk

### The application-side SDK for AxonOS brain–computer interfaces.

<sub>Typed intent events · capability declarations · AxonOS Consent Protocol integration · `no_std`-capable · `#![deny(unsafe_code)]`</sub>

<br/>

[![Crate](https://img.shields.io/badge/Crate-v0.3.5-0a4a8f?style=flat-square)](https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.3.5)
[![Standard](https://img.shields.io/badge/Standard-v1.0.0-0a4a8f?style=flat-square)](https://github.com/AxonOS-org/axonos-standard)
[![Kernel ABI](https://img.shields.io/badge/Kernel%20ABI-v1-0a4a8f?style=flat-square)](#compatibility-with-the-kernel)
[![Rust](https://img.shields.io/badge/Rust-no__std-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Unsafe](https://img.shields.io/badge/Unsafe-denied-0d7a5f?style=flat-square)](#security)
[![License](https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-475569?style=flat-square)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-475569?style=flat-square)](https://blog.rust-lang.org/2023/12/28/Rust-1.85.0.html)
[![Target](https://img.shields.io/badge/Target-Cortex--M%20%2F%20host-475569?style=flat-square)](https://doc.rust-lang.org/rustc/platform-support/thumbv7em-none-eabi.html)
[![AxonOS Radar](https://img.shields.io/endpoint?url=https%3A%2F%2Faxonos-bci.github.io%2Faxonos-community-radar%2Fbadges%2FAxonOS-org%2Faxonos-sdk.json&style=flat-square)](https://axonos-bci.github.io/axonos-community-radar/)

[**About**](./ABOUT.md) &nbsp;·&nbsp; [**Modules**](#modules) &nbsp;·&nbsp; [**Quick start**](#quick-start) &nbsp;·&nbsp; [**Security**](./SECURITY.md) &nbsp;·&nbsp; [**Contributing**](./CONTRIBUTING.md) &nbsp;·&nbsp; [**License**](#license)

</div>

---

## In one paragraph

`axonos-sdk` is the **application-side counterpart** to
[`axonos-kernel`](https://github.com/AxonOS-org/axonos-kernel). The
kernel runs the real-time signal pipeline on a Cortex-M microcontroller
and emits typed intent observations through a strict RFC-0006 wire
format and a capability gate. The SDK is what an application links to
in order to **read** those observations, **declare** which capability
classes it requires, and **integrate** with the AxonOS Consent
Protocol. It has no `unsafe` code, no allocator on the hot path, and
compiles on the same Cortex-M targets as the kernel.

## What this crate gives you

- **`IntentObservation`** — the 32-byte, 8-byte-aligned in-memory record
  matching kernel RFC-0006. Construct synthetic events for testing and read
  their typed kind, timestamp and confidence; decoding from raw wire bytes is
  not in this crate yet (see *What does not exist yet* below).
- **`Manifest`** — declare your application's required capability set
  at compile time. The kernel rejects manifests larger than the
  catalogue at construction, so policy mismatches are caught early.
- **`MonotonicTimestamp`** — portable monotonic-microsecond type with
  WCET-documented arithmetic. Saturating, never panicking.
- **`Stream`** — typed iterator-style API for consuming a sequence of
  intent observations from an IPC source.
- **`Host`** — host-side helpers for testing application code without
  a physical Cortex-M board.
- **`Mesh`** — multi-node coordination primitives (`MeshClientStub`)
  matching the [`axonos-swarm`](https://github.com/AxonOS-org/axonos-swarm)
  protocol. Stub client today; full transport tracked for Phase 2.

## Modules

| Module | Purpose | `no_std` ok |
|:---|:---|:---:|
| `intent` | RFC-0006 wire format, Q0.16 confidence, IntentObservation | ✓ |
| `capability` | Capability enum, manifest declaration, isolation gate | ✓ |
| `manifest` | Compile-time manifest builder, validation | ✓ |
| `time` | `MonotonicTimestamp`, saturating arithmetic | ✓ |
| `stream` | Typed observation stream over an IPC source | ✓ |
| `error` | Exhaustive `SdkError` enum | ✓ |
| `host` | Host-side test helpers | `std` only |
| `mesh` | Multi-node coordination (`MeshClientStub`) | ✓ |
| `zerocopy_ext` | Zero-copy IPC helpers (feature-gated, internal) | ✓ |

## Quick start

`axonos-sdk` is not published on crates.io; depend on a tagged release:

```toml
[dependencies]
axonos-sdk = { git = "https://github.com/AxonOS-org/axonos-sdk", tag = "v0.3.5" }
```

```rust
use axonos_sdk::{Capability, Direction, IntentKind, IntentObservation, Manifest, MonotonicTimestamp};

fn main() -> Result<(), axonos_sdk::Error> {
    // 1. Declare what your application needs. Nothing else will be granted.
    let manifest = Manifest::builder()
        .app_id("com.example.cursor")?
        .capability(Capability::Navigation)
        .max_rate_hz(50)
        .build()?;
    assert!(manifest.allows(Capability::Navigation));
    assert!(!manifest.allows(Capability::WorkloadAdvisory));

    // 2. Work with the 32-byte observation type. Until the kernel ABI ships,
    //    observations are constructed locally, e.g. in tests.
    let obs = IntentObservation::new_direction(
        MonotonicTimestamp::from_micros_unchecked(1_000),
        Direction::Up,
        58_982, // Q0.16 confidence, v / 65535 ≈ 0.90
        7,
        [0u8; 8],
    );
    if let IntentKind::Direction(d) = obs.kind() {
        println!("{d:?} @ {} µs, confidence raw {}", obs.timestamp_us(), obs.confidence_raw());
    }
    Ok(())
}
```

**What does not exist yet.** There is no transport from a running kernel to
an application: `IntentStream::connect` (feature `std`) succeeds only against
the in-memory test fixture, and `IntentStream::try_next` exists only under the
development-only `kernel-stub` feature, where it always returns `None`. Both
arrive with the kernel ABI. The SDK also has no decoder from raw wire bytes;
the byte-exact codec for the RFC-0006 record lives in
[`axonos-conformance`](https://github.com/AxonOS-org/axonos-conformance) and
in `axonos-kernel`'s `axonos-intent` crate.

## Feature flags

| Feature | Purpose | Default |
|:---|:---|:---:|
| `std` | Enable host-side helpers, `thiserror` errors | off |
| `alloc` | Use `alloc` for dynamic buffers (still no `std`) | off |
| `serde` | Derive `Serialize`/`Deserialize` on observation types | off |
| `kernel-stub` | **Development only** — link against a stub kernel for tests | off |

## Compatibility with the kernel

The SDK consumes the wire format defined in [`axonos-kernel`](https://github.com/AxonOS-org/axonos-kernel)
`axonos-intent` crate. Both implement RFC-0006 §4.1 independently — two
implementations cross-validate one another. The SDK's
`IntentObservation::decode` and the kernel's
`axonos_intent::IntentObservation::encode` round-trip through the
network on conformance vectors.

ABI compatibility is tracked via `KERNEL_ABI_VERSION`:

| ABI | Kernel | SDK |
|:---:|:---|:---|
| **v1** | `axonos-kernel ≥ 0.1.6` | `axonos-sdk ≥ 0.3.0` |

## Stability

This crate is pre-1.0. The wire format (RFC-0006) is **frozen**.
The Rust API may evolve before 1.0 — breaking changes will be
documented in [CHANGELOG.md](./CHANGELOG.md).

**Current limitations (honest status):**

- Real kernel transport is not yet wired. `IntentStream::try_next`
  requires the `kernel-stub` feature (returns `Ok(None)`) until L3
  validation lands, tracked in RFC-0005 — pending an instrumented evaluation-board fixture, which is not yet procured; no date is given because one would be invented.
- L3 oscilloscope-validated WCRT is **pending**. Performance figures in
  this SDK are stated at L1 or L2 per the validation taxonomy.
- No same-hardware controlled benchmark against other RTOS platforms
  has been performed.

## Security

- `#![deny(unsafe_code)]` across the entire crate. The single audited
  `unsafe` module is gated behind a future feature flag and not active
  in v0.1.x.
- No allocator on the hot path. All collections are static-sized.
- Mutex poison handling: SDK's sync primitives never panic on poisoned
  state — they return a structured `SdkError::PoisonRecovery`.
- Fixed-point Q0.16 confidence: bit-identical across all targets
  (x86_64 SSE, Cortex-M4F FPU, soft-float). No floating-point in the
  hot path.

Report security issues to [security@axonos.org](mailto:security@axonos.org).

## Forking

Forking is welcomed and the procedure takes three clicks. See
[CONTRIBUTING.md](./CONTRIBUTING.md) for the post-fork compliance
burden (which is small). Apache-2.0 OR MIT licensing is permissive —
use, modify, redistribute, commercialise.

## Repository structure

```
axonos-sdk/
├── README.md                    ← this file
├── ABOUT.md                     ← purpose, audience, market
├── CONTRIBUTING.md              ← fork in 3 clicks
├── ENTERPRISE.md                ← commercial support tiers
├── SECURITY.md                  ← vulnerability-disclosure policy
├── CHANGELOG.md
├── CITATION.cff
├── NOTICE                       ← Apache-2.0 attribution + trademark
├── LICENSE-APACHE
├── LICENSE-MIT
├── Cargo.toml
├── deny.toml · rustfmt.toml
├── .github/workflows/           ← ci.yml, release.yml
├── docs/
│   └── SECURITY-AUDIT.md        ← independent audit finding→fix record
├── src/
│   ├── lib.rs
│   ├── intent.rs                ← RFC-0006 wire format, Q0.16 confidence
│   ├── capability.rs            ← Capability enum, manifest gate
│   ├── manifest.rs              ← Manifest builder, validation
│   ├── time.rs                  ← MonotonicTimestamp
│   ├── stream.rs                ← Typed observation stream
│   ├── error.rs                 ← SdkError
│   ├── host.rs                  ← Host-side test helpers (std)
│   ├── mesh.rs                  ← Multi-node coordination (stub client)
│   └── zerocopy_ext.rs          ← Zero-copy IPC helpers (feature-gated)
└── benches/
    └── intent_throughput.rs
```

## License

Dual-licensed at your option under:

- **[Apache License, Version 2.0](./LICENSE-APACHE)**
- **[MIT License](./LICENSE-MIT)**

See [NOTICE](./NOTICE) for Apache-2.0 required attribution and the
trademark policy. Contributions are accepted under the inbound = outbound
model (no separate CLA) — see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Position in the AxonOS stack

| Layer | Repository | Role |
|---|---|---|
| Canonical standard | [`axonos-standard`](https://github.com/AxonOS-org/axonos-standard) | Architecture manual, conformance criteria, validation taxonomy |
| Engineering RFCs | [`axonos-rfcs`](https://github.com/AxonOS-org/axonos-rfcs) | Numbered design proposals (RFC-0001 through RFC-0006) |
| Kernel substrate | [`axonos-kernel`](https://github.com/AxonOS-org/axonos-kernel) | Real-time pipeline; emits the wire format this SDK consumes |
| **Application boundary** | **`axonos-sdk`** | Typed intents, manifests, ABI-compatible integration |
| Consent layer | [`axonos-consent`](https://github.com/AxonOS-org/axonos-consent) | Deterministic consent state machine and stimulation-gating protocol |
| Mesh coordination | [`axonos-swarm`](https://github.com/AxonOS-org/axonos-swarm) | Distributed timing, co-availability, peer health monitoring |

- **Project website:** [axonos.org](https://axonos.org).
- **Long-form essays:** [medium.com/@AxonOS](https://medium.com/@AxonOS).

---

<div align="center">

**The AxonOS Project** &nbsp;·&nbsp; [axonos.org](https://axonos.org) &nbsp;·&nbsp; [connect@axonos.org](mailto:connect@axonos.org) &nbsp;·&nbsp; [security@axonos.org](mailto:security@axonos.org)

[medium.com/@AxonOS](https://medium.com/@AxonOS) &nbsp;·&nbsp; [github.com/AxonOS-org](https://github.com/AxonOS-org)

<sub>© 2026 Denis Yermakou · `axonos-sdk` v0.3.5</sub>

</div>
