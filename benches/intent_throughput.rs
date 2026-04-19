// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Criterion benchmarks for SDK hot paths.
//!
//! Run with:
//!
//! ```sh
//! cargo bench --features std
//! ```

use axonos_sdk::{
    Capability, Direction, IntentKind, IntentObservation, ObservationFilter,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_observation_construction(c: &mut Criterion) {
    c.bench_function("IntentObservation::new_direction", |b| {
        b.iter(|| {
            IntentObservation::new_direction(
                black_box(1_000),
                black_box(Direction::Up),
                black_box(0.85),
                black_box(42),
                black_box([0u8; 8]),
            )
        });
    });
}

fn bench_observation_kind_decode(c: &mut Criterion) {
    let obs = IntentObservation::new_direction(0, Direction::Right, 0.9, 1, [0; 8]);
    c.bench_function("IntentObservation::kind", |b| {
        b.iter(|| {
            let k = black_box(&obs).kind();
            debug_assert!(matches!(k, IntentKind::Direction(_)));
            k
        });
    });
}

fn bench_filter_match(c: &mut Criterion) {
    let f = ObservationFilter::MinConfidence(u16::MAX / 2);
    let obs = IntentObservation::new_direction(0, Direction::Up, 0.9, 0, [0; 8]);
    c.bench_function("ObservationFilter::matches", |b| {
        b.iter(|| black_box(&f).matches(black_box(&obs)));
    });
}

fn bench_capability_set_insert(c: &mut Criterion) {
    c.bench_function("CapabilitySet::with", |b| {
        b.iter(|| {
            axonos_sdk::CapabilitySet::new()
                .with(black_box(Capability::Navigation))
                .with(black_box(Capability::SessionQuality))
        });
    });
}

criterion_group!(
    benches,
    bench_observation_construction,
    bench_observation_kind_decode,
    bench_filter_match,
    bench_capability_set_insert,
);
criterion_main!(benches);
