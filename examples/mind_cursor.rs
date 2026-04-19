// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Motor-imagery cursor example.
//!
//! Demonstrates a realistic BCI application loop: receive directional
//! intents, apply a confidence-gated exponential smoothing filter, and
//! update a simulated 2D cursor position.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example mind_cursor --features "std serde"
//! ```

use axonos_sdk::{
    host::InMemoryFixture, Capability, Direction, IntentKind, IntentObservation, IntentStream,
    Manifest, ObservationFilter,
};

/// Minimum confidence for a direction to move the cursor.
const CONFIDENCE_THRESHOLD: f32 = 0.60;
/// Smoothing factor (0.0 = no smoothing, 1.0 = no history).
const SMOOTHING: f32 = 0.3;
/// Cursor step size in pixels.
const STEP_PX: f32 = 8.0;

#[derive(Default, Debug, Clone, Copy)]
struct Cursor {
    x: f32,
    y: f32,
}

impl Cursor {
    fn apply(&mut self, dx: f32, dy: f32) {
        self.x = SMOOTHING.mul_add(dx, (1.0 - SMOOTHING) * self.x);
        self.y = SMOOTHING.mul_add(dy, (1.0 - SMOOTHING) * self.y);
    }
}

fn direction_vector(d: Direction) -> (f32, f32) {
    match d {
        Direction::Up => (0.0, -STEP_PX),
        Direction::Down => (0.0, STEP_PX),
        Direction::Left => (-STEP_PX, 0.0),
        Direction::Right => (STEP_PX, 0.0),
        Direction::Neutral => (0.0, 0.0),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::builder()
        .app_id("com.axonos.example.cursor")?
        .name("Mind Cursor")?
        .capability(Capability::Navigation)
        .capability(Capability::SessionQuality)
        .max_rate_hz(50)
        .build()?;

    // Install fixture: mixed-confidence observations.
    let mut fx = InMemoryFixture::new();
    for (t, d, c) in [
        (10_000, Direction::Up, 0.85),
        (30_000, Direction::Up, 0.35),   // low confidence, should be filtered
        (50_000, Direction::Right, 0.80),
        (70_000, Direction::Right, 0.75),
        (90_000, Direction::Neutral, 0.50), // below threshold, ignored
    ] {
        fx.push(IntentObservation::new_direction(t, d, c, 42, [0; 8]));
    }
    fx.install();

    // The confidence threshold is enforced client-side here.
    // In a production application, consider asking the kernel for a
    // higher-confidence classifier tier instead of client-side filtering.
    let filter = ObservationFilter::MinConfidence(
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        ((CONFIDENCE_THRESHOLD * f32::from(u16::MAX)) as u16),
    );

    let mut stream = IntentStream::connect(&manifest)?;
    let mut cursor = Cursor::default();

    println!("Mind Cursor — consuming 5 observations");
    println!("Confidence threshold: {:.0}%", CONFIDENCE_THRESHOLD * 100.0);
    println!();

    for i in 0..5 {
        if let Some(obs) = stream.try_next()? {
            let passes_filter = filter.matches(&obs);
            if let IntentKind::Direction(d) = obs.kind() {
                let (dx, dy) = direction_vector(d);
                if passes_filter {
                    cursor.apply(dx, dy);
                    println!(
                        "#{i}: {:?} [{:.0}%] → cursor=({:.1}, {:.1})",
                        d,
                        obs.confidence() * 100.0,
                        cursor.x,
                        cursor.y,
                    );
                } else {
                    println!(
                        "#{i}: {:?} [{:.0}%] filtered (below threshold)",
                        d,
                        obs.confidence() * 100.0,
                    );
                }
            }
        }
    }

    Ok(())
}
