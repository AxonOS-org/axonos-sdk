// SPDX-License-Identifier: Apache-2.0 OR MIT
//! End-to-end integration tests for the AxonOS SDK.
//!
//! These tests exercise the full public API surface using the
//! `InMemoryFixture` host transport.

#![cfg(feature = "std")]

use axonos_sdk::host::InMemoryFixture;
use axonos_sdk::{
    Capability, Direction, Error, IntentKind, IntentObservation, IntentStream, Manifest,
    ObservationFilter, OverflowPolicy, Quality,
};

fn test_manifest(app: &str) -> Manifest {
    Manifest::builder()
        .app_id(app)
        .unwrap()
        .capability(Capability::Navigation)
        .capability(Capability::SessionQuality)
        .max_rate_hz(10)
        .build()
        .unwrap()
}

#[test]
fn connect_with_fixture_and_manifest_succeeds() {
    let mut fx = InMemoryFixture::new();
    fx.push(IntentObservation::new_direction(100, Direction::Up, 0.9, 1, [0; 8]));
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
        .app_id("com.test.e2e.3")
        .unwrap()
        .capability(Capability::WorkloadAdvisory) // kernel limit 1 Hz
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
fn observation_kind_round_trips_for_all_variants() {
    // Direction
    for d in [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::Neutral,
    ] {
        let obs = IntentObservation::new_direction(0, d, 0.7, 0, [0; 8]);
        assert_eq!(obs.kind(), IntentKind::Direction(d));
    }
    // Quality
    for q in [
        Quality::High,
        Quality::Moderate,
        Quality::Low,
        Quality::NoSignal,
    ] {
        let obs = IntentObservation::new_quality(0, q, 0, [0; 8]);
        assert_eq!(obs.kind(), IntentKind::Quality(q));
    }
}

#[test]
fn filter_rejects_low_confidence() {
    let high_bar = u16::MAX - 1000; // ~97%
    let f = ObservationFilter::MinConfidence(high_bar);
    let low = IntentObservation::new_direction(0, Direction::Up, 0.3, 0, [0; 8]);
    let high = IntentObservation::new_direction(0, Direction::Up, 0.99, 0, [0; 8]);
    assert!(!f.matches(&low));
    assert!(f.matches(&high));
}

#[test]
fn error_is_terminal_classification_matches_docs() {
    // These are the advertised terminal errors.
    assert!(Error::ConsentWithdrawn.is_terminal());
    assert!(Error::AttestationFailed.is_terminal());
    assert!(Error::AbiMismatch { sdk: 1, kernel: 2 }.is_terminal());

    // And the non-terminal set.
    assert!(!Error::ConsentSuspended.is_terminal());
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
    // Bits 0 and 1.
    assert_eq!(s.as_raw(), 0b0000_0011);
}

#[test]
fn observation_size_is_32_bytes() {
    assert_eq!(std::mem::size_of::<IntentObservation>(), 32);
}

#[test]
fn version_constants_surface_correctly() {
    assert_eq!(axonos_sdk::MMP_CONSENT_VERSION, "0.1.0");
    assert!(axonos_sdk::KERNEL_ABI_VERSION >= 1);
    assert!(!axonos_sdk::VERSION.is_empty());
}
