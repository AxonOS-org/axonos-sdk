<div align="center">

<img src="https://rustacean.net/assets/rustacean-flat-happy.svg" width="120" alt="Ferris, the Rust mascot" />

# axonos-sdk

### the application-side SDK for AxonOS brain–computer interfaces

> Typed intent events, capability declarations, AxonOS Consent Protocol integration. `no_std`-capable. `#![deny(unsafe_code)]`. The consumer half of the AxonOS kernel substrate.

[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0%20OR%20MIT-blue?style=for-the-badge)](#license)
[![no_std](https://img.shields.io/badge/no__std-yes-success?style=for-the-badge)](https://docs.rust-embedded.org/book/intro/no-std.html)
[![Safety-critical](https://img.shields.io/badge/safety-critical-red?style=for-the-badge)](SECURITY.md)

[![MSRV](https://img.shields.io/badge/MSRV-1.75-orange?style=flat-square)](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)
[![deny unsafe](https://img.shields.io/badge/unsafe-deny-brightgreen?style=flat-square)](https://doc.rust-lang.org/reference/attributes/codegen.html)
[![Cortex-M](https://img.shields.io/badge/embedded-Cortex--M-purple?style=flat-square)](https://doc.rust-lang.org/rustc/platform-support/thumbv7em-none-eabi.html)
[![Kernel ABI v1](https://img.shields.io/badge/ABI-v1-yellow?style=flat-square)](#stability)

[**About**](./ABOUT.md) · [**Modules**](#modules) · [**Quick start**](#quick-start) · [**Security**](./SECURITY.md) · [**Contributing**](./CONTRIBUTING.md) · [**License**](#license)

</div>

---

## In one paragraph

`axonos-sdk` is the **application-side counterpart** to
[`axonos-kernel`](https://github.com/AxonOS-org/AxonOS-kernel). The
kernel runs the real-time signal pipeline on a Cortex-M microcontroller
and emits typed intent observations through a strict RFC-0006 wire
format and a capability gate. The SDK is what an application links to
in order to **read** those observations, **declare** which capability
classes it requires, and **integrate** with the AxonOS Consent
Protocol. It has no `unsafe` code, no allocator on the hot path, and
compiles on the same Cortex-M targets as the kernel.

## What this crate gives you

- **`IntentObservation`** — the 32-byte, 8-byte-aligned wire record
  matching kernel RFC-0006. Decode incoming bytes into a typed event;
  encode synthetic events for testing.
- **`Manifest`** — declare your application's required capability set
  at compile time. The kernel rejects manifests larger than the
  catalogue at construction, so policy mismatches are caught early.
- **`MonotonicTimestamp`** — portable monotonic-microsecond type with
  WCET-documented arithmetic. Saturating, never panicking.
- **`Stream`** — typed iterator-style API for consuming a sequence of
  intent observations from an IPC source.
- **`Host`** — host-side helpers for testing application code without
  a physical Cortex-M board.
- **`Mesh`** — multi-node coordination primitives matching the
  `axonos-swarm` protocol (forthcoming).
- **`Telemetry`** — opt-in, capability-gated EEG/EMG buffer parsing.
- **`FFI`** — C/C++/Python bindings for non-Rust application code.

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
| `mesh` | Multi-node coordination | ✓ |
| `telemetry` | EEG/EMG parsing (stub — Phase 2) | ✓ |
| `ffi` | C/C++/Python bindings (stub — Phase 2) | `std` only |
| `zerocopy_ext` | Zero-copy extensions for the IPC buffer | ✓ |

## Quick start

```toml
[dependencies]
axonos-sdk = "0.1"
```

```rust,no_run
use axonos_sdk::intent::IntentObservation;
use axonos_sdk::capability::{Capability, CapabilitySet};
use axonos_sdk::manifest::Manifest;

// 1. Declare what your application needs.
let manifest = Manifest::new(
    CapabilitySet::singleton(Capability::Navigation)
        .with(Capability::SessionQuality),
);

// 2. Decode an incoming observation from the kernel.
let bytes: [u8; 32] = receive_from_kernel_ipc();
let observation = IntentObservation::decode(&bytes)?;

// 3. Check capability before acting on the event.
if manifest.contains(observation.kind.capability()) {
    handle_intent(&observation);
}
```

## Feature flags

| Feature | Purpose | Default |
|:---|:---|:---:|
| `std` | Enable host-side helpers, `thiserror` errors | off |
| `alloc` | Use `alloc` for dynamic buffers (still no `std`) | off |
| `serde` | Derive `Serialize`/`Deserialize` on observation types | off |
| `kernel-stub` | **Development only** — link against a stub kernel for tests | off |

## Compatibility with the kernel

The SDK consumes the wire format defined in [`axonos-kernel`](https://github.com/AxonOS-org/AxonOS-kernel)
`axonos-intent` crate. Both implement RFC-0006 §4.1 independently — two
implementations cross-validate one another. The SDK's
`IntentObservation::decode` and the kernel's
`axonos_intent::IntentObservation::encode` round-trip through the
network on conformance vectors.

ABI compatibility is tracked via `KERNEL_ABI_VERSION`:

| ABI | Kernel | SDK |
|:---:|:---|:---|
| **v1** | `axonos-kernel ≥ 0.1.6` | `axonos-sdk ≥ 0.1.0` |

## Stability

This crate is pre-1.0. The wire format (RFC-0006) is **frozen**.
The Rust API may evolve before 1.0 — breaking changes will be
documented in [CHANGELOG.md](./CHANGELOG.md).

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
├── NOTICE                       ← Apache-2.0 attribution
├── LICENSE-APACHE
├── LICENSE-MIT
├── SECURITY.md                  ← disclosure policy
├── CHANGELOG.md
├── Cargo.toml
├── .github/workflows/ci.yml
├── src/
│   ├── lib.rs
│   ├── intent.rs                ← RFC-0006 wire format
│   ├── capability.rs            ← Capability enum
│   ├── manifest.rs              ← Manifest builder
│   ├── time.rs                  ← MonotonicTimestamp
│   ├── stream.rs                ← Typed observation stream
│   ├── error.rs                 ← SdkError
│   ├── host.rs                  ← Host-side helpers
│   ├── mesh.rs                  ← Multi-node coordination
│   ├── telemetry.rs             ← EEG/EMG (stub)
│   ├── ffi.rs                   ← C/C++/Python bindings (stub)
│   └── zerocopy_ext.rs          ← Zero-copy IPC
├── examples/
│   ├── bare_metal_no_std.rs
│   └── mesh_coupling.rs
├── benches/
│   └── intent_throughput.rs
└── tests/
```

## License

Dual-licensed at your option under:

- **[Apache License, Version 2.0](./LICENSE-APACHE)**
- **[MIT License](./LICENSE-MIT)**

See [NOTICE](./NOTICE) for Apache-2.0 required attribution and the
trademark policy. Contributions are accepted under the inbound = outbound
model (no separate CLA) — see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Related

- **[`axonos-kernel`](https://github.com/AxonOS-org/AxonOS-kernel)** —
  the verifiable kernel substrate. The SDK consumes its wire format.
- **[`axonos-rfcs`](https://github.com/AxonOS-org/axonos-rfcs)** —
  engineering specifications (RFC-0001 through RFC-0006).
- **Project website:** [axonos.org](https://axonos.org).
- **Long-form essays:** [medium.com/@AxonOS](https://medium.com/@AxonOS).

---

<div align="center">

**Author and maintainer:** Denis Yermakou · [denis@axonos.org](mailto:denis@axonos.org)

Zurich · Berlin · Milano · San Mateo · Singapore

<sub>Made with 🦀 and a long real-time tick.</sub>

</div>
