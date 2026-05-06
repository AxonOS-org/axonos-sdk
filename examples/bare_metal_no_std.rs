// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `no_std` demonstration — no allocation, all-stack.
//!
//! ```sh
//! cargo run --example bare_metal_no_std --features kernel-stub
//! cargo build --target thumbv7em-none-eabihf --no-default-features --features kernel-stub
//! ```

use axonos_sdk::{
    Capability, CapabilitySet, Direction, IntentKind, IntentObservation, Manifest,
    MonotonicTimestamp,
};

fn main() {
    demo();
    println!("bare-metal demo completed — no allocations, all-stack");
}

fn demo() {
    let manifest = Manifest::builder()
        .app_id("embedded.demo")
        .capability(Capability::Navigation)
        .max_rate_hz(10)
        .build()
        .expect("static manifest construction should not fail");

    let caps: CapabilitySet = *manifest.capabilities();
    assert!(caps.contains(Capability::Navigation));

    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    let obs = IntentObservation::new_direction(ts, Direction::Up, 58982, 0, [0u8; 8]);

    match obs.kind() {
        IntentKind::Direction(Direction::Up) => println!("received Up"),
        other => println!("received {other:?}"),
    }

    println!(" IntentObservation: {} bytes", core::mem::size_of_val(&obs));
    println!(" Manifest: {} bytes", core::mem::size_of_val(&manifest));
    println!(" CapabilitySet: {} bytes", core::mem::size_of_val(&caps));
    println!(" MonotonicTimestamp: {} bytes", core::mem::size_of::<MonotonicTimestamp>());
}
