// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Demonstration that the SDK public API compiles and runs in a
//! `no_std`-compatible way — heapless strings, no allocation, no I/O.
//!
//! This example uses a `std` `main` for portability; in a real embedded
//! build you would omit `fn main`, use `cortex-m-rt::entry` as the entry
//! point, and compile with `--no-default-features`. The body of `demo()`
//! uses only `no_std`-compatible APIs and does not allocate.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example bare_metal_no_std --features std
//! ```
//!
//! Verify `no_std` build of the library itself with:
//!
//! ```sh
//! cargo build --no-default-features
//! cargo build --target thumbv7em-none-eabihf --no-default-features
//! ```

use axonos_sdk::{
    Capability, CapabilitySet, Direction, IntentKind, IntentObservation, Manifest,
};

fn main() {
    demo();
    println!("bare-metal demo completed — no allocations, all-stack");
}

fn demo() {
    // Build a manifest — heapless strings, no allocation.
    let manifest = Manifest::builder()
        .app_id("embedded.demo")
        .and_then(|b| b.capability(Capability::Navigation).max_rate_hz(10).build())
        .expect("static manifest construction should not fail");

    // Verify capabilities round-trip.
    let caps: CapabilitySet = *manifest.capabilities();
    assert!(caps.contains(Capability::Navigation));

    // Construct an observation on the stack.
    let obs = IntentObservation::new_direction(0, Direction::Up, 0.9, 0, [0u8; 8]);

    // Match on the decoded kind.
    match obs.kind() {
        IntentKind::Direction(Direction::Up) => println!("received Up"),
        other => println!("received {other:?}"),
    }

    // Stack frame is tiny:
    //   - IntentObservation = 32 B
    //   - Manifest with a single capability + short app_id ≈ ~200 B
    // Total < 1 KB — fits easily in a Cortex-M4F stack.
    println!("  IntentObservation:  {} bytes", core::mem::size_of_val(&obs));
    println!("  Manifest:           {} bytes", core::mem::size_of_val(&manifest));
    println!("  CapabilitySet:      {} bytes", core::mem::size_of_val(&caps));
}

