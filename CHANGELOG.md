# Notable changes

All notable changes to `axonos-sdk` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## 2026-05-13 — Security audit hardening (breaking changes)

### Summary

- **`ManifestBuilder` setters return `Result<Self, Error>` instead of panicking.**
  `assert!` on input length has been replaced with `Err(ManifestRejected::Malformed)`.
  In `panic = "abort"` builds, a malformed app_id no longer terminates the process.

- **`MonotonicTimestamp::elapsed_since` returns `Option<u64>` instead of `u64`.**
  Silent return of 0 on clock violation could mask kernel bugs in release builds.
  Use [`elapsed_since_saturating`](MonotonicTimestamp::elapsed_since_saturating)
  if a clamped value is genuinely desired.

- **`MonotonicTimestamp::Deserialize` validates input.**
  Values exceeding `SESSION_MAX_REASONABLE_US` (2^48 µs ≈ 8.9 years) are rejected.
  Prevents adversarial network input from corrupting downstream WCET arithmetic.

- **`InMemoryFixture::install`, `uninstall` return `Result<()>`.**
  Poisoned mutex now surfaces as `TransportFault::Internal` rather than
  silent recovery via `into_inner()`.

- **`CapabilitySet::with` uses `wrapping_shl` for defence-in-depth.**
  Even if `CAPABILITY_COUNT` is incorrectly raised past 32 in future,
  no undefined behaviour occurs.

- **`IntentStream::try_next` gated by `#[cfg(feature = "kernel-stub")]`.**
  Replaces the previous in-body `compile_error!` hack with proper cfg gating.
  Calling code without the feature now produces a cleaner build error.

### Added

- `MonotonicTimestamp::from_micros_validated(us)` — validated constructor.
- `MonotonicTimestamp::elapsed_since_saturating(earlier)` — explicit saturating
  variant for cases where 0-on-violation is the intended response.
- `Capability::from_u8(v)` — fallible discriminant constructor.
- `Capability::bit()` — centralised bit-position helper.
- `CapabilityIter` — zero-cost custom iterator (4-state machine, no array allocation).
- `impl IntoIterator for CapabilitySet` and `impl IntoIterator for &CapabilitySet`.
- `TransportFault::Internal` — variant for poisoned mutex / corrupted state.

### Changed

- `host::resolve_endpoint` returns `Cow<'static, str>` (avoids allocation for default).
- `zerocopy_ext` is now `pub(crate)` (reduced public surface).
- `examples/mind_cursor.rs` no longer uses `f32::mul_add` (FPU-only on bare-metal).

### Removed

- All references to the previous external protocol coupling.
  AxonOS Consent Protocol (ACP) is the sole consent specification.

### Migration guide

```diff
- let m = Manifest::builder()
-     .app_id("com.example.app")
-     .capability(Capability::Navigation)
-     .build()?;
+ let m = Manifest::builder()
+     .app_id("com.example.app")?
+     .capability(Capability::Navigation)
+     .max_rate_hz(10)
+     .build()?;

- let elapsed: u64 = t2.elapsed_since(t1);
+ let elapsed: u64 = t2.elapsed_since(t1)
+     .ok_or(Error::ClockViolation)?;
+ // Or, if 0-on-violation is the intended response:
+ let elapsed: u64 = t2.elapsed_since_saturating(t1);

- fx.install();
+ fx.install()?;
```

---

## 2026-04-22 — Production-hardening release

- Production-hardened SDK release.
- `#![deny(unsafe_code)]` enforced.
- `compile_error!` on unimplemented security paths.
- `MeshClientStub` renamed to clarify stub status.
- WCET documentation for `MonotonicTimestamp` operations.

---

## 2026-04-15 — Initial public release

- Initial public release.
- Capability declarations, typed intent events, consent integration.
