// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Minimal example: connect, subscribe, print.
//!
//! ```sh
//! cargo run --example hello_intent --features "std kernel-stub"
//! ```

use axonos_sdk::{
    host::InMemoryFixture, Capability, Direction, IntentKind, IntentObservation, IntentStream,
    Manifest, MonotonicTimestamp,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::builder()
        .app_id("com.axonos.example.hello")?
        .name("Hello Intent")?
        .vendor("AxonOS")?
        .capability(Capability::Navigation)
        .max_rate_hz(10)
        .build()?;

    println!("Manifest for {} ({:?})", manifest.app_id(), manifest.name());

    let mut fx = InMemoryFixture::new();
    let ts = MonotonicTimestamp::from_micros_unchecked;
    fx.push(IntentObservation::new_direction(
        ts(1_000),
        Direction::Up,
        58982,
        1,
        [0; 8],
    ));
    fx.push(IntentObservation::new_direction(
        ts(2_000),
        Direction::Right,
        55704,
        1,
        [0; 8],
    ));
    fx.push(IntentObservation::new_direction(
        ts(3_000),
        Direction::Down,
        51099,
        1,
        [0; 8],
    ));
    fx.install();

    let mut stream = IntentStream::connect(&manifest)?;
    println!("Connected. Listening for 3 observations...");

    for _ in 0..3 {
        if let Some(obs) = stream.try_next()? {
            match obs.kind() {
                IntentKind::Direction(d) => {
                    println!(
                        "[{:>6} µs] direction={:?} confidence_raw={} (~{:.0}%)",
                        obs.timestamp_us(),
                        d,
                        obs.confidence_raw(),
                        obs.confidence_f32() * 100.0
                    );
                }
                other => println!("[{:>6} µs] {:?}", obs.timestamp_us(), other),
            }
        }
    }

    Ok(())
}
