// SPDX-License-Identifier: Apache-2.0 OR MIT
//! End-to-end integration tests.

#![cfg(feature = "std")]

use axonos_sdk::host::InMemoryFixture;
use axonos_sdk::{
    Capability, Direction, Error, IntentKind, IntentObservation, IntentStream, Manifest,
    MonotonicTimestamp, ObservationFilter, OverflowPolicy, Quality,
};

fn test_manifest(app: &str) -> Manifest {
    Manifest::builder()
        .app_id(app)?
        .capability(Capability::Navigation)
        .capability(Capability::SessionQuality)
        .max_rate_hz(10)
        .build()
        .unwrap()
}

#[test]
fn connect_with_fixture_and_manifest_succeeds() {
    let mut fx = InMemoryFixture::new();
    let ts = MonotonicTimestamp::from_micros_unchecked;
    fx.push(IntentObservation::new_direction(ts(100), Direction::Up, 58982, 1, [0; 8]));
    fx.install();

    let manifest = test_manifest("com.test.e2e.1");
    let stream = IntentStream::connect(&manifest).unwrap();
    assert!(stream.is_connected());

    InMemoryFixture::uninstall();
}

#[test]
fn connect_without_fixture_fails_with_transport_error() {
    InMemoryFixture::uninstall();
    let manifest = test_manifest("com.test.e2e.2");
    let r = IntentStream::connect(&manifest);
    assert!(matches!(r, Err(Error::TransportUnreachable(_))));
}

#[test]
fn manifest_rejects_rate_over_kernel_limit() {
    let r = Manifest::builder()
        .app_id("com.test.e2e.3")?
        .capability(Capability::WorkloadAdvisory)
        .max_rate_hz(100)
        .build();
    assert!(r.is_err());
    if let Err(Error::ManifestRejected { reason }) = r {
        use axonos_sdk::error::ManifestRejection;
        assert_eq!(reason, ManifestRejection::RateTooHigh);
    } else {
        panic!("expected ManifestRejected");
    }
}

#[test]
fn manifest_rejects_zero_rate() {
    let r = Manifest::builder()
        .app_id("com.test.e2e.zero")?
        .capability(Capability::Navigation)
        .max_rate_hz(0)
        .build();
    assert!(r.is_err());
}

#[test]
fn observation_kind_round_trips_for_all_variants() {
    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    for d in [Direction::Up, Direction::Right, Direction::Down, Direction::Left, Direction::Neutral] {
        let obs = IntentObservation::new_direction(ts, d, 32768, 0, [0; 8]);
        assert_eq!(obs.kind(), IntentKind::Direction(d));
    }
    for q in [Quality::High, Quality::Moderate, Quality::Low, Quality::NoSignal] {
        let obs = IntentObservation::new_quality(ts, q, 0, [0; 8]);
        assert_eq!(obs.kind(), IntentKind::Quality(q));
    }
}

#[test]
fn filter_rejects_low_confidence() {
    let high_bar = 64000u16;
    let f = ObservationFilter::MinConfidence(high_bar);
    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    let low = IntentObservation::new_direction(ts, Direction::Up, 1000, 0, [0; 8]);
    let high = IntentObservation::new_direction(ts, Direction::Up, 65000, 0, [0; 8]);
    assert!(!f.matches(&low));
    assert!(f.matches(&high));
}

#[test]
fn error_is_terminal_classification_matches_docs() {
    assert!(Error::ConsentWithdrawn.is_terminal());
    assert!(!Error::ConsentSuspended.is_terminal());
    assert!(Error::AttestationFailed.is_terminal());
    assert!(!Error::StreamOverflow { dropped: 10 }.is_terminal());
}

#[test]
fn overflow_policy_default_is_drop_oldest() {
    assert_eq!(OverflowPolicy::default(), OverflowPolicy::DropOldest);
}

#[test]
fn capability_set_bitfield_is_compact() {
    use axonos_sdk::CapabilitySet;
    let s = CapabilitySet::new()
        .with(Capability::Navigation)
        .with(Capability::WorkloadAdvisory);
    assert_eq!(s.as_raw().as_u32(), 0b0000_0011);
}

#[test]
fn capability_set_u32_width_prevents_overflow() {
    use axonos_sdk::CapabilitySet;
    let s = CapabilitySet::new()
        .with(Capability::Navigation)
        .with(Capability::WorkloadAdvisory)
        .with(Capability::SessionQuality)
        .with(Capability::ArtifactEvents);
    assert_eq!(s.len(), 4);
    assert_eq!(s.as_raw().as_u32(), 0x0F);
}

#[test]
fn observation_size_is_32_bytes() {
    assert_eq!(std::mem::size_of::<IntentObservation>(), 32);
}

#[test]
fn observation_align_is_8() {
    assert_eq!(std::mem::align_of::<IntentObservation>(), 8);
}

#[test]
fn version_constants_surface_correctly() {
    assert_eq!(axonos_sdk::CONSENT_PROTOCOL_VERSION, "0.2.0");
    assert!(axonos_sdk::KERNEL_ABI_VERSION >= 1);
    assert!(!axonos_sdk::VERSION.is_empty());
}

#[test]
fn manifest_builder_is_infallible_intermediate() {
    let _builder = Manifest::builder()
        .app_id("com.test.builder")?
        .name("Test")?
        .vendor("AxonOS")?
        .capability(Capability::Navigation)
        .max_rate_hz(10);
    assert!(_builder.build().is_ok());
}

#[test]
fn fixed_point_confidence_is_deterministic() {
    let ts = MonotonicTimestamp::from_micros_unchecked(0);
    let obs = IntentObservation::new_direction(ts, Direction::Up, 32768, 0, [0u8; 8]);
    assert_eq!(obs.confidence_raw(), 32768);
    let f = obs.confidence_f32();
    assert!(f > 0.499 && f < 0.501);
}

#[test]
fn monotonic_timestamp_arithmetic() {
    let t1 = MonotonicTimestamp::from_micros_unchecked(1000);
    let t2 = MonotonicTimestamp::from_micros_unchecked(2500);
    assert_eq!(t2.checked_sub(t1), Some(1500));
    assert_eq!(t1.checked_sub(t2), None);
    assert_eq!(t2.as_millis(), 2);
}

#[test]
fn capability_set_display_format() {
    use axonos_sdk::CapabilitySet;
    let s = CapabilitySet::new()
        .with(Capability::Navigation)
        .with(Capability::SessionQuality);
    let formatted = format!("{}", s);
    assert!(formatted.contains("navigation"));
    assert!(formatted.contains("session_quality"));
}

#[test]
fn raw_capability_set_detects_reserved_bits() {
    use axonos_sdk::RawCapabilitySet;
    let raw = RawCapabilitySet(0x0F);
    assert!(!raw.has_reserved_bits());
    let raw_bad = RawCapabilitySet(0xFF);
    assert!(raw_bad.has_reserved_bits());
}
