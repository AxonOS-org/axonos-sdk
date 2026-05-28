# ABOUT — AxonOS SDK

## What this is

`axonos-sdk` is the **application-side counterpart** to
[`axonos-kernel`](https://github.com/AxonOS-org/axonos-kernel). It is
the library an application links against in order to consume intent
observations emitted by the AxonOS real-time kernel running on a
microcontroller.

Where the kernel handles deadline-bound signal processing, capability
isolation, and the wire format on the silicon side, the SDK gives
application code a typed, ergonomic, safe interface to read what the
kernel produced and to declare what it is permitted to read.

This crate is `#![deny(unsafe_code)]`, `no_std`-capable, and compiles
on the same Cortex-M targets as the kernel.

## For whom this is written

| Audience | What they will find here |
|:---|:---|
| **Application developers** building closed-loop neural assistive interfaces | A typed `IntentObservation` decoder, a compile-time `Manifest` declaration, and an iterator-style `Stream` API over the kernel's IPC output. |
| **Research scientists** writing custom BCI experiments | A drop-in SDK that integrates with the kernel without requiring you to re-implement RFC-0006 wire-format parsing or capability handshaking. |
| **Tooling and benchmark authors** | A `host` module with non-`no_std` test helpers and stable `IntentObservation` builders for offline analysis and replay. |
| **Non-Rust integrators** (C/C++/Python applications) | An `ffi` module (Phase 2) exposing the same data model through a stable C ABI. |
| **Multi-device coordination** (clinics, swarm experiments) | A `mesh` module aligned with the forthcoming `axonos-swarm` protocol. |

## What problem it addresses

Application code that consumes a real-time neural classifier output
has historically had to:

1. Re-parse a bespoke binary wire format per device.
2. Re-implement capability gating in application code.
3. Re-write integration boilerplate for every new BCI platform.

`axonos-sdk` does all three once, against the RFC-0006 wire format,
in safe Rust, so that applications can ship their domain logic
without owning the substrate.

## What it is not

- **It is not the kernel.** The kernel lives in
  [`axonos-kernel`](https://github.com/AxonOS-org/axonos-kernel) and
  runs on the microcontroller. The SDK runs in your application.
- **It is not a signal-processing library.** It receives classified
  intent events from the kernel; it does not classify EEG itself.
- **It is not a medical device.** This is application-side software
  that may participate in a medical device system subject to the
  appropriate regulatory process. No clinical claims attach to this
  repository as published.
- **It is not the AI/inference layer.** The kernel's
  classifier and any application-side intelligence are separate
  concerns; the SDK is the typed boundary between them.

## Status, in plain terms

- **Code:** 12 source modules, no_std-capable, `#![deny(unsafe_code)]`.
- **Maturity:** Pre-1.0. The wire format (RFC-0006) is frozen; the
  Rust API may evolve.
- **Tests:** Host-side unit tests, conformance vectors against the
  kernel's `axonos-intent` crate.
- **ABI version:** v1 (KERNEL_ABI_VERSION constant), compatible with `axonos-kernel ≥ 0.1.9`.
- **Integration:** Phase 2 (Q3 2026) — full integration with the
  reference firmware on STM32F407.

## How to engage

The SDK is open source under dual Apache-2.0 / MIT licensing. The
preferred way to engage is to read the code, run the examples,
file an issue or RFC contribution.

- **Security disclosures:** `security@axonos.org`
- **Technical correspondence:** `connect@axonos.org`
- **Partnership / clinical engagement:** `connect@axonos.org`

---

**Author:** Denis Yermakou · connect@axonos.org · [axonos.org](https://axonos.org)
