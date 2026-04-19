// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `no_std` bare-metal demonstration — shows how the SDK compiles without
//! the `std` feature.
//!
//! This example uses `#[cfg(feature = "std")]` to gate `main`; in a real
//! embedded build the application would define its own entry point using
//! `cortex-m-rt` or equivalent.
//!
//! Run with:
//!
//! ```sh
//! cargo build --example bare_metal_no_std --no-default-features
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[cfg(feature = "std")]
fn main() {
    demo();
}

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    demo();
    loop {}
}

fn demo() {
    use axonos_sdk::{
        Capability, CapabilitySet, Direction, IntentKind, IntentObservation, Manifest,
    };

    // Build a manifest — heapless strings, no allocation.
    let manifest_result = Manifest::builder()
        .app_id("embedded.demo")
        .and_then(|b| b.capability(Capability::Navigation).max_rate_hz(10).build());

    let manifest = match manifest_result {
        Ok(m) => m,
        Err(_) => return, // cannot recover in bare-metal; in production use a panic handler
    };

    // Verify capabilities round-trip.
    let caps: CapabilitySet = *manifest.capabilities();
    let _ = caps.contains(Capability::Navigation);

    // Construct an observation on the stack.
    let obs = IntentObservation::new_direction(0, Direction::Up, 0.9, 0, [0u8; 8]);

    // Match on the decoded kind.
    match obs.kind() {
        IntentKind::Direction(Direction::Up) => {
            #[cfg(feature = "std")]
            println!("received Up");
        }
        _ => {
            #[cfg(feature = "std")]
            println!("other");
        }
    }

    // Stack frame is tiny:
    //   - IntentObservation = 32 B
    //   - Manifest with a single capability + short app_id ≈ ~200 B
    // Total < 1 KB — fits easily in a Cortex-M4F stack.
    let _ = core::mem::size_of_val(&obs);
    let _ = core::mem::size_of_val(&manifest);
}
