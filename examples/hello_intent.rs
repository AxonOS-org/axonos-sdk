// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Minimal example: connect, subscribe to navigation intents, print them.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example hello_intent --features std
//! ```

use axonos_sdk::{
    host::InMemoryFixture, Capability, Direction, IntentKind, IntentObservation, IntentStream,
    Manifest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── Build a manifest declaring Navigation capability ──────────────
    let manifest = Manifest::builder()
        .app_id("com.axonos.example.hello")?
        .name("Hello Intent")?
        .vendor("AxonOS")?
        .capability(Capability::Navigation)
        .max_rate_hz(10)
        .build()?;

    println!("Manifest for {} ({:?})", manifest.app_id(), manifest.name());
    println!("  capabilities: {:?}", manifest.capabilities());

    // ── Install a scripted fixture for this example ─────────────────
    let mut fx = InMemoryFixture::new();
    fx.push(IntentObservation::new_direction(1_000, Direction::Up, 0.9, 1, [0; 8]));
    fx.push(IntentObservation::new_direction(2_000, Direction::Right, 0.85, 1, [0; 8]));
    fx.push(IntentObservation::new_direction(3_000, Direction::Down, 0.78, 1, [0; 8]));
    fx.install();

    // ── Connect to the local kernel ─────────────────────────────────
    let mut stream = IntentStream::connect(&manifest)?;
    println!("Connected. Listening for 3 observations...");

    // In a real application this loop runs indefinitely. For the example
    // we exit after draining the fixture.
    for _ in 0..3 {
        if let Some(obs) = stream.try_next()? {
            match obs.kind() {
                IntentKind::Direction(d) => {
                    println!(
                        "[{:>6} µs] direction={:?} confidence={:.0}%",
                        obs.timestamp_us(),
                        d,
                        obs.confidence() * 100.0
                    );
                }
                other => println!("[{:>6} µs] {:?}", obs.timestamp_us(), other),
            }
        }
    }

    Ok(())
}
