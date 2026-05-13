// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Motor-imagery cursor with confidence-gated smoothing.
//!
//! ```sh
//! cargo run --example mind_cursor --features "std serde kernel-stub"
//! ```

use axonos_sdk::{
    host::InMemoryFixture, Capability, Direction, IntentKind, IntentObservation, IntentStream,
    Manifest, MonotonicTimestamp, ObservationFilter,
};

/// Q0.16 threshold: 65535 * 0.60 = 39321
const CONFIDENCE_THRESHOLD_RAW: u16 = 39321;
const SMOOTHING: f32 = 0.3;
const STEP_PX: f32 = 8.0;

#[derive(Default, Debug, Clone, Copy)]
struct Cursor { x: f32, y: f32 }

impl Cursor {
    fn apply(&mut self, dx: f32, dy: f32) {
        self.x = SMOOTHING * dx + (1.0 - SMOOTHING) * self.x;
        self.y = SMOOTHING * dy + (1.0 - SMOOTHING) * self.y;
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

    let mut fx = InMemoryFixture::new();
    let ts = MonotonicTimestamp::from_micros_unchecked;
    let observations = [
        (10_000, Direction::Up, 58982u16),
        (30_000, Direction::Up, 22937u16),
        (50_000, Direction::Right, 52428u16),
        (70_000, Direction::Right, 49151u16),
        (90_000, Direction::Neutral, 32768u16),
    ];
    for (t, d, c) in observations {
        fx.push(IntentObservation::new_direction(ts(t), d, c, 42, [0; 8]));
    }
    fx.install();

    let filter = ObservationFilter::MinConfidence(CONFIDENCE_THRESHOLD_RAW);
    let mut stream = IntentStream::connect(&manifest)?;
    let mut cursor = Cursor::default();

    println!("Mind Cursor — 5 observations");
    println!("Threshold: {} raw (~{:.0}%)", CONFIDENCE_THRESHOLD_RAW,
        (CONFIDENCE_THRESHOLD_RAW as f32 / 65535.0) * 100.0);
    println!();

    for i in 0..5 {
        if let Some(obs) = stream.try_next()? {
            let passes = filter.matches(&obs);
            if let IntentKind::Direction(d) = obs.kind() {
                let (dx, dy) = direction_vector(d);
                if passes {
                    cursor.apply(dx, dy);
                    println!("#{i}: {:?} [raw={}] → ({:.1}, {:.1})", d, obs.confidence_raw(), cursor.x, cursor.y);
                } else {
                    println!("#{i}: {:?} [raw={}] filtered", d, obs.confidence_raw());
                }
            }
        }
    }

    Ok(())
}
