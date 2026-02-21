# AxonOS SDK 🧠⚙️

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](#)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](#)
[![no_std](https://img.shields.io/badge/no__std-supported-success)](#)

The official Rust Software Development Kit for **AxonOS** — a cognitive operating system bridging Artificial Intelligence and Brain-Computer Interfaces (BCI).

This SDK provides the foundational data structures, traits, and Foreign Function Interface (FFI) bindings required to seamlessly connect hardware (EEG/EMG gateways) to the AxonOS bare-metal kernel and the upper AI/ML layers.

---

## 🏗 Architecture & Modules

The SDK is designed for zero-overhead abstraction, hard real-time constraints, and absolute memory safety. It is split into three core components:

### 1. The `BciStream` Trait (Core Interface)
At the heart of the SDK is the `BciStream` trait. Any hardware driver or BCI gateway must implement this trait to securely stream neural telemetry into the AxonOS kernel. It ensures deterministic polling and safe error handling.

### 2. `telemetry` Module (Signal Parsing)
Designed for high-frequency biosignal streams.
* Utilizes zero-copy deserialization (`bincode`) to parse incoming raw byte streams from sensors.
* Defines the strict memory layouts for EEG/EMG packets required by the kernel's Cognitive Scheduler.

### 3. `ffi` Module (Cross-Boundary Execution)
AxonOS operates on a bare-metal Rust kernel, but modern AI models rely on Python and C++. The `ffi` module provides safe, zero-cost bindings:
* **C-API:** Exported headers for low-level device drivers.
* **Python Interop:** Safe data transfer to machine learning pipelines without heavy context switching.

---

## 🚀 Quick Start: Implementing a BCI Gateway

To build a custom driver for a new neuro-headset, simply implement the `BciStream` trait. 

```rust
use axonos_sdk::BciStream;
use axonos_sdk::telemetry::EegPacket;

pub struct MyCustomHeadset {
    is_connected: bool,
}

impl BciStream for MyCustomHeadset {
    type Error = core::fmt::Error;

    fn begin_stream(&mut self) -> Result<(), Self::Error> {
        // 1. Initialize hardware registers
        // 2. Begin DMA transfer of raw signal
        // 3. Route to AxonOS kernel
        Ok(())
    }
}
🛠 Compilation Features
This crate is highly modular. You can tailor it to your target environment using Cargo features:

no_std by default: The core crate builds without the standard library, making it ready for embedded and bare-metal targets.

async: Enables asynchronous stream processing via tokio (requires standard library).

c-ffi: Generates C-compatible headers for the SDK.

python-interop: Enables PyO3 bindings for direct integration with AI models in Python.

📦 Building from Source
To compile the SDK for a bare-metal environment (Release mode):
cargo build --release --no-default-features
To compile as a dynamic library (.so / .dylib) for FFI integration:
cargo build --release --features "c-ffi"
📜 License
This project is dual-licensed under the MIT and Apache 2.0 licenses.
See the LICENSE files for details.

Developed by the AxonOS Engineering Team.
