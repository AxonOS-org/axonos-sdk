<div align="center">

# AxonOS SDK

### Build BCI applications on a deterministic cognitive OS

Public Rust SDK for the AxonOS platform — typed `IntentObservation` events, capability-based neural permissions, and WASM sandbox host bindings. For developers building on the first bare-metal microkernel designed for brain-computer interfaces.

[![Crates.io](https://img.shields.io/crates/v/axonos-sdk?style=for-the-badge&logo=rust&color=dea584)](https://crates.io/crates/axonos-sdk)
[![docs.rs](https://img.shields.io/docsrs/axonos-sdk?style=for-the-badge&logo=docs.rs)](https://docs.rs/axonos-sdk)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=for-the-badge)](#licence)
[![CI](https://img.shields.io/github/actions/workflow/status/AxonOS-org/axonos-sdk/ci.yml?style=for-the-badge&logo=github)](https://github.com/AxonOS-org/axonos-sdk/actions)

[![Rust](https://img.shields.io/badge/Rust-2021-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-orange?style=flat-square)](https://github.com/AxonOS-org/axonos-sdk/blob/main/Cargo.toml)
[![FFI](https://img.shields.io/badge/FFI-C%20%7C%20Python-informational?style=flat-square)](#language-bindings)
[![async](https://img.shields.io/badge/async-tokio-8A2BE2?style=flat-square)](https://tokio.rs/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=flat-square)]()
[![no\_alloc](https://img.shields.io/badge/RT%20path-zero%20alloc-success?style=flat-square)](#zero-allocation-guarantee)

[Website](https://axonos.org) · [Documentation](https://docs.rs/axonos-sdk) · [Examples](examples/) · [Medium](https://medium.com/@AxonOS) · [LinkedIn](https://www.linkedin.com/in/axonos)

</div>

---

## What this SDK is

The AxonOS kernel processes 8-channel EEG on bare-metal ARM Cortex-M33 with a verified worst-case execution time of **618 µs** and **2.4 µs RMS jitter** — see [Benchmark Report](https://medium.com/@AxonOS/axonos-mvp-the-benchmark-report-latency-power-ea6c78d0e091). The kernel is `#![no_std]` Rust running in TrustZone Secure World, with no path from application code to the underlying neural signal.

This SDK is the **public API surface** third-party developers build against:

- Typed `IntentObservation` events — the only output kernel produces
- Capability manifest authoring and validation
- WASM host bindings for the application sandbox
- Async intent stream (tokio) for cognitive app development
- C FFI bindings for C/C++/Python integration
- Zero-copy telemetry parsing (serde + bincode)

> **What this SDK is not.** It is not the kernel. The AxonOS microkernel, DSP pipeline, and TrustZone partition are in private repositories during the pre-release phase. This SDK defines what applications see — the kernel implements the other side of the contract.

---

## Install

```toml
[dependencies]
axonos-sdk = "0.1"
```

With feature flags:

```toml
[dependencies]
axonos-sdk = { version = "0.1", features = ["async", "c-ffi"] }
```

| Feature | Default | Description |
|:---|:---:|:---|
| `async` | ✓ | Tokio runtime integration for non-blocking intent streams |
| `c-ffi` | ✓ | C header generation via cbindgen, `#[no_mangle]` exports |
| `python-interop` | — | PyO3 bindings for Python ML layer (experimental) |

---

## Quickstart

### Reading intent observations

```rust
use axonos_sdk::{IntentStream, IntentKind, ObservationFilter};

#[tokio::main]
async fn main() -> Result<(), axonos_sdk::Error> {
    // Connect to AxonOS kernel via NSC gateway
    let stream = IntentStream::connect("axonos://local").await?;

    // Filter: only motor imagery, confidence > 0.85
    let filter = ObservationFilter::new()
        .kind(IntentKind::NavigationIntent)
        .min_confidence(0.85);

    let mut observations = stream.subscribe(filter).await?;

    while let Some(obs) = observations.next().await {
        println!(
            "Intent: {:?} (confidence: {:.2}, t: {} µs)",
            obs.intent_id, obs.posterior, obs.timestamp_us
        );
        // HMAC-SHA256 tag verified by kernel before delivery
    }
    Ok(())
}
```

### Declaring capabilities

Every AxonOS application ships a signed manifest. Capabilities an application does not declare **do not exist** in its WASM execution environment — enforcement is structural, not policy-based.

```toml
# axonos-app.toml
[app]
name = "mind-cursor"
version = "1.0.0"
signer = "ed25519:MCowBQYDK2VwAyEA..."

[capabilities]
NavigationIntents = { min_confidence = 0.80 }
SessionQuality   = { }
# RawEEG, EmotionState, CognitiveProfile — NOT requestable
```

```rust
use axonos_sdk::manifest::{Manifest, Capability};

let manifest = Manifest::load("axonos-app.toml")?;
manifest.verify_signature()?;
let caps: Vec<Capability> = manifest.capabilities();
```

---

## Core types

### `IntentObservation`

The only event type the kernel emits. Every observation is cryptographically attested and bounded to a capability the application has declared.

| Field | Type | Description |
|:---|:---|:---|
| `intent_id` | `IntentKind` | Typed discriminant — see [capability classes](#capability-classes) |
| `posterior` | `f32` | Posterior probability in `[0.0, 1.0]` (Q16 internally) |
| `timestamp_us` | `u64` | Microsecond-precision kernel timestamp |
| `session_id` | `[u8; 16]` | Opaque session identifier (not biometric) |
| `hmac_sha256` | `[u8; 32]` | Attestation tag, verified by kernel pre-delivery |

### `IntentKind`

```rust
pub enum IntentKind {
    NavigationIntent(Direction),  // 85–95% accuracy, motor imagery 2-class
    WorkloadAdvisory(Load),       // ~70% accuracy, binary cognitive load
    SessionQuality(Quality),      // discrete: signal-to-noise, electrode contact
    ArtifactEvent(ArtifactType),  // EMG, eye-blink, motion artifacts
}
```

---

## Capability classes

Applications declare capabilities at install time. The kernel binds only declared capabilities to the WASM instance — unrequested functions are absent from the execution environment.

### Available

| Capability | Accuracy | Granularity |
|:---|:---:|:---|
| `NavigationIntents` | 85–95% | Motor imagery 2-class (left/right) |
| `WorkloadAdvisory` | ~70% | Binary cognitive load (high/low) |
| `SessionQuality` | Discrete | Electrode contact, signal-to-noise |
| `ArtifactEvents` | Event-based | EMG, eye-blink, motion artifacts |

### Not requestable

| Capability | Reason |
|:---|:---|
| `RawEEG` | Architectural boundary — no API exposed to Non-Secure World |
| `EmotionState` | 60–70% accuracy, insufficient for informed consent |
| `FlowState` | Not reliably detectable in real-time |
| `CognitiveProfile` | Prohibited by design |

Full architecture: [Articles #3, #7, #9, #10](https://medium.com/@AxonOS) on Medium.

---

## Language bindings

### C / C++

Headers auto-generated via [cbindgen](https://github.com/mozilla/cbindgen) during `cargo build`:

```c
#include <axonos_sdk.h>

int main(void) {
    axonos_stream_t* stream = axonos_stream_connect("axonos://local");
    axonos_observation_t obs;
    while (axonos_stream_next(stream, &obs, 100 /*ms timeout*/) == AXONOS_OK) {
        printf("Intent: %d, confidence: %.2f\n", obs.intent_id, obs.posterior);
    }
    axonos_stream_free(stream);
    return 0;
}
```

### Python (via PyO3)

```python
import axonos_sdk as ax

stream = ax.IntentStream.connect("axonos://local")
stream.set_filter(min_confidence=0.85, kind="navigation")

for obs in stream:
    print(f"Intent: {obs.intent_id}, conf: {obs.posterior:.2f}")
```

Build Python bindings:

```bash
cargo build --release --features python-interop
maturin build --release --features python-interop
```

---

## Protocol integration

AxonOS implements the [**Mesh Memory Protocol (MMP) v0.2.2**](https://sym.bot/spec/mmp) for multi-node BCI deployments. The SDK exposes peer-to-peer cognitive coupling through a type-safe wrapper:

```rust
use axonos_sdk::mesh::{MeshClient, ConsentScope};

let mesh = MeshClient::new("wss://sym-relay.onrender.com").await?;

// Withdraw consent from specific peer — triggers hardware DAC gate closure
// per MMP Consent Extension v0.1.0, typically <10 µs end-to-end
mesh.withdraw_consent(peer_id, ConsentScope::Peer).await?;
```

The Consent Extension enforces consent at **Layer 2** (Connection), below the SVAF coupling engine ([arXiv:2604.03955](https://arxiv.org/abs/2604.03955)). Reference implementation: [axonos-consent](https://github.com/AxonOS-org/axonos-consent).

---

## Zero-allocation guarantee

All real-time paths in this SDK are allocation-free. The `IntentObservation` type implements `Copy` (32 bytes, Plain Old Data). The async stream uses a bounded `tokio::sync::mpsc` channel with a fixed capacity — no unbounded queues, no heap growth under backpressure.

```rust
use axonos_sdk::{IntentStream, StreamConfig};

let config = StreamConfig::default()
    .channel_capacity(64)       // bounded, compile-time-sized-looking
    .overflow(OverflowPolicy::DropOldest);  // explicit, never block sender

let stream = IntentStream::with_config(config).await?;
```

The `serde` + `bincode` deserialization uses zero-copy slice borrows where possible — `Observation<'a>` holds references into the original buffer, no heap copy.

---

## Performance characteristics

Measured on x86_64 Linux, Rust 1.75, release profile (LTO fat, codegen-units = 1):

| Operation | p50 | p99 | Notes |
|:---|:---:|:---:|:---|
| `IntentObservation::verify_hmac` | 1.1 µs | 2.3 µs | Software SHA-256 |
| `Manifest::verify_signature` | 85 µs | 120 µs | Ed25519, one-shot at load |
| `IntentStream::next` (tokio) | 0.3 µs | 0.9 µs | Channel receive, no allocation |
| C FFI entry/exit overhead | 12 ns | 28 ns | `#[no_mangle]` extern "C" |

Kernel-side timing (Cortex-M33 @ 120 MHz): **618 µs WCET** pipeline, **2.4 µs RMS jitter**. See [Article #12 — Benchmark Report](https://medium.com/@AxonOS/axonos-mvp-the-benchmark-report-latency-power-ea6c78d0e091).

---

## Error handling

This SDK follows Rust library conventions — `thiserror` for typed errors, no `anyhow` (which would leak error types to callers):

```rust
use axonos_sdk::Error;

match stream.next().await {
    Ok(obs) => handle(obs),
    Err(Error::HmacMismatch) => {
        // Attestation failed — kernel detected tampering
        log::error!("Consent revoked: attestation failure");
    }
    Err(Error::ConsentWithdrawn { peer_id }) => {
        // MMP Consent Extension — peer withdrew
        log::info!("Peer {} withdrew consent", peer_id);
    }
    Err(Error::CapabilityDenied { requested, granted }) => {
        // Manifest did not declare this capability
        log::warn!("Need {requested:?}, have {granted:?}");
    }
    Err(e) => log::error!("Stream error: {}", e),
}
```

---

## Examples

| Example | Description |
|:---|:---|
| [`examples/hello_intent.rs`](examples/hello_intent.rs) | Minimal intent subscriber |
| [`examples/mind_cursor.rs`](examples/mind_cursor.rs) | Motor imagery → screen cursor mapping |
| [`examples/focus_monitor.rs`](examples/focus_monitor.rs) | Cognitive load telemetry with privacy-preserving aggregation |
| [`examples/mesh_coupling.rs`](examples/mesh_coupling.rs) | Multi-node BCI via MMP with consent withdrawal |
| [`examples/c_embedding/`](examples/c_embedding/) | C/C++ integration with cbindgen header |
| [`examples/python_bridge/`](examples/python_bridge/) | Python ML inference over PyO3 |

Run:

```bash
cargo run --example hello_intent
cargo run --example mesh_coupling --features async
```

---

## AxonOS ecosystem

| Repository | Role |
|:---|:---|
| [`axonos-sdk`](https://github.com/AxonOS-org/axonos-sdk) | **This crate** — public API for applications |
| [`axonos-consent`](https://github.com/AxonOS-org/axonos-consent) | MMP Consent Extension (Rust `no_std` reference impl) |
| [`axon-bci-gateway`](https://github.com/AxonOS-org/axon-bci-gateway) | OpenBCI GUI fork for hardware bring-up |
| `axonos-kernel` | Bare-metal microkernel (private, pre-release) |
| `axonos-dsp` | CSP + MDM + Kalman signal processing (private) |
| `axonos-sim` | Hardware-in-the-loop simulator (private) |

---

## Documentation & research

| Resource | Link |
|:---|:---|
| API reference | [docs.rs/axonos-sdk](https://docs.rs/axonos-sdk) |
| Website | [axonos.org](https://axonos.org) |
| Engineering series (30 articles) | [medium.com/@AxonOS](https://medium.com/@AxonOS) |
| Benchmark report | [Article #12](https://medium.com/@AxonOS/axonos-mvp-the-benchmark-report-latency-power-ea6c78d0e091) |
| Protocol collaboration | [Article #39 — MMP with SYM.BOT](https://medium.com/@AxonOS) |
| SVAF paper (coupling layer) | [arXiv:2604.03955](https://arxiv.org/abs/2604.03955) |

---

## Minimum Supported Rust Version

Rust **1.75** (stable). MSRV bumps are treated as **minor** version bumps of this crate.

---

## Contributing

Pre-release phase — the public API surface is stabilising. Contributions welcome for:

- Language bindings (Swift, Kotlin, TypeScript via WASM)
- Example applications demonstrating capability-bounded neural app patterns
- Documentation improvements and translations
- Integration tests against the SDK's public API

Core kernel, DSP pipeline, and TrustZone partition contributions are not accepted via this repo — those live in private pre-release repositories. Open an issue to discuss architecture-level proposals.

See [CONTRIBUTING.md](CONTRIBUTING.md) and the [project security policy](SECURITY.md).

---

## Licence

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option. Unless explicitly stated otherwise, any contribution intentionally submitted for inclusion in this work shall be dual licensed as above, without any additional terms or conditions.

---

<div align="center">

**AxonOS. Pure signal. Zero noise.**

[axonos.org](https://axonos.org) · [medium.com/@AxonOS](https://medium.com/@AxonOS) · [linkedin.com/in/axonos](https://www.linkedin.com/in/axonos) · [axonosorg@gmail.com](mailto:axonosorg@gmail.com)

</div>
