// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Focus monitor: aggregate cognitive load events into a 1-minute
//! privacy-preserving average.
//!
//! This example demonstrates the `WorkloadAdvisory` capability — a
//! deliberately low-bandwidth channel (≤1 Hz) that avoids sending
//! continuous mental-state data.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example focus_monitor --features std
//! ```

use axonos_sdk::{
    host::InMemoryFixture, Capability, IntentKind, IntentObservation, IntentStream, Load, Manifest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::builder()
        .app_id("com.axonos.example.focus")?
        .name("Focus Monitor")?
        .capability(Capability::WorkloadAdvisory)
        .capability(Capability::SessionQuality)
        .max_rate_hz(1) // respect the 1 Hz kernel limit for WorkloadAdvisory
        .build()?;

    // Fixture: 5 workload samples at 1 Hz.
    let mut fx = InMemoryFixture::new();
    for (i, load) in [Load::Low, Load::Moderate, Load::Moderate, Load::High, Load::Moderate]
        .iter()
        .enumerate()
    {
        #[allow(clippy::cast_possible_truncation)]
        fx.push(IntentObservation::new_load(
            (i as u64) * 1_000_000,
            *load,
            0.80,
            7,
            [0; 8],
        ));
    }
    fx.install();

    let mut stream = IntentStream::connect(&manifest)?;

    let mut score_sum: i32 = 0;
    let mut samples: i32 = 0;

    for _ in 0..5 {
        if let Some(obs) = stream.try_next()? {
            if let IntentKind::Load(load) = obs.kind() {
                let score = match load {
                    Load::Low => 1,
                    Load::Moderate => 2,
                    Load::High => 3,
                };
                score_sum += score;
                samples += 1;
                println!(
                    "[t={:>4}s] load={:?} (score={}) conf={:.0}%",
                    obs.timestamp_us() / 1_000_000,
                    load,
                    score,
                    obs.confidence() * 100.0,
                );
            }
        }
    }

    if samples > 0 {
        let avg = f64::from(score_sum) / f64::from(samples);
        println!();
        println!("Average load over {} samples: {:.2} (1=low, 3=high)", samples, avg);
    }

    Ok(())
}
