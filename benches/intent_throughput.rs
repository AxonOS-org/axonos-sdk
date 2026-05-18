// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Criterion benchmarks.
//!
//! ```sh
//! cargo bench --features "std kernel-stub"
//! ```

use axonos_sdk::{
    Capability, Direction, IntentKind, IntentObservation, MonotonicTimestamp, ObservationFilter,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_observation_construction(c: &mut Criterion) {
    let ts = MonotonicTimestamp::from_micros_unchecked;
    c.bench_function("IntentObservation::new_direction", |b| {
        b.iter(|| {
            IntentObservation::new_direction(
                black_box(ts(1_000)),
                black_box(Direction::Up),
                black_box(58982u16),
                black_box(42),
                black_box([0u8; 8]),
            )
        });
    });
}

fn bench_observation_kind_decode(c: &mut Criterion) {
    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    let obs = IntentObservation::new_direction(ts, Direction::Right, 58982, 1, [0; 8]);
    c.bench_function("IntentObservation::kind", |b| {
        b.iter(|| {
            let k = black_box(&obs).kind();
            debug_assert!(matches!(k, IntentKind::Direction(_)));
            k
        });
    });
}

fn bench_filter_match(c: &mut Criterion) {
    let f = ObservationFilter::MinConfidence(32768);
    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    let obs = IntentObservation::new_direction(ts, Direction::Up, 58982, 0, [0; 8]);
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

fn bench_monotonic_timestamp_ops(c: &mut Criterion) {
    let t1 = MonotonicTimestamp::from_micros_unchecked(1_000_000);
    let t2 = MonotonicTimestamp::from_micros_unchecked(2_500_000);
    c.bench_function("MonotonicTimestamp::checked_sub", |b| {
        b.iter(|| black_box(t2).checked_sub(black_box(t1)));
    });
}

criterion_group!(
    benches,
    bench_observation_construction,
    bench_observation_kind_decode,
    bench_filter_match,
    bench_capability_set_insert,
    bench_monotonic_timestamp_ops,
);
criterion_main!(benches);

