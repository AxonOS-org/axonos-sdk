# Release Notes — axonos-sdk 0.4.0

Release date: 2026-05-13

## Summary

This release implements all findings from an independent Rust security
audit conducted in May 2026 (14 items across 4 severity levels) and
completes the removal of references to the legacy external protocol
coupling — the SDK is now self-contained around the AxonOS Consent
Protocol (ACP) v0.2.0.

This is a **breaking release**. Public API surface in `ManifestBuilder`,
`MonotonicTimestamp`, and `InMemoryFixture` has changed. A migration
guide is included in [CHANGELOG.md](./CHANGELOG.md).

---

## What's new

### 🔴 Critical (P0) — public API hardening

Three fixes that close gaps where adversarial or malformed input could
cross safety boundaries.

**`ManifestBuilder` setters are fallible.**
Every setter that validates input length now returns
`Result<Self, Error>` instead of `assert!`-ing on malformed input.
In `panic = "abort"` builds — typical for embedded targets — a
malformed `app_id` no longer terminates the application process.

```rust
// before (0.3.x)
let m = Manifest::builder()
    .app_id("com.example")              // could panic on bad input
    .build()?;

// after (0.4.0)
let m = Manifest::builder()
    .app_id("com.example")?             // returns Result
    .build()?;
```

**`MonotonicTimestamp::elapsed_since` returns `Option<u64>`.**
The previous signature returned `0` silently on clock violation
(`earlier > self`), which could mask kernel bugs in release builds.
Callers now decide explicitly how to handle violations.

```rust
// before
let elapsed: u64 = t2.elapsed_since(t1);  // silently 0 on violation

// after
let elapsed: u64 = t2.elapsed_since(t1)
    .ok_or(MyError::ClockViolation)?;

// or, if 0-on-violation is the intentional response:
let elapsed: u64 = t2.elapsed_since_saturating(t1);
```

**`MonotonicTimestamp::Deserialize` validates bounds.**
Values exceeding `SESSION_MAX_REASONABLE_US` (2⁴⁸ µs ≈ 8.9 years)
are rejected. Prevents adversarial CBOR/JSON input from corrupting
downstream WCET arithmetic.

### 🟠 High priority (P1) — defence in depth

- `CapabilitySet::with` uses `wrapping_shl` — even if `CAPABILITY_COUNT`
  is incorrectly raised past 32 in future without updating the
  compile-time guard, no undefined behaviour occurs.
- `IntentObservation` has compile-time assertions enforcing
  `size == 32` and `align == 8` (matches the wire format committed
  in RFC-0006).
- `InMemoryFixture::install` / `uninstall` return `Result<()>`.
  Poisoned mutex surfaces as `TransportFault::Internal` rather than
  silent `into_inner()` recovery.
- Per-capability rate limit is now documented in `max_rate_hz`: the
  effective rate for capability `c` is
  `min(max_rate_hz, c.kernel_rate_limit_hz())`.

### 🟡 Medium priority (P2) — performance and ergonomics

- `CapabilityIter` is a zero-cost custom 4-state iterator
  (previously `[Capability; 4].into_iter().filter(...)`, which
  generated a fat iterator type).
- `host::resolve_endpoint` returns `Cow<'static, str>` — no allocation
  when the default endpoint is used.
- `TransportFault::Internal` variant added for poisoned-mutex /
  corrupted-state errors.
- `examples/mind_cursor.rs` no longer uses `f32::mul_add`, which
  required hardware FPU and would have invoked software emulation
  (and WCET inflation) on soft-float targets.

### 🟢 Low priority (P3) — surface and ergonomics

- `zerocopy_ext` is now `pub(crate)` — reduces the public surface area
  carrying `#[allow(unsafe_code)]` before 1.0.
- `IntentStream::try_next` is gated by `#[cfg(feature = "kernel-stub")]`
  on the function itself, replacing the in-body `compile_error!` hack.
- `impl IntoIterator for CapabilitySet` and `impl IntoIterator for
  &CapabilitySet` — `for c in cap_set` and `for c in &cap_set` both
  work now.

### 🧹 Cleanup

All references to the previous external protocol coupling have been
removed across `.rs`, `.toml`, and `.md` files. The SDK now refers
solely to the AxonOS Consent Protocol (ACP), specified in
`axonos-consent` v0.4.0 and RFC-0001 / RFC-0006 of `axonos-rfcs`.

---

## Migration guide

| 0.3.x                              | 0.4.0                                    |
|:-----------------------------------|:-----------------------------------------|
| `.app_id("…")`                     | `.app_id("…")?`                          |
| `.name("…")`                       | `.name("…")?`                            |
| `.vendor("…")`                     | `.vendor("…")?`                          |
| `t2.elapsed_since(t1) -> u64`      | `t2.elapsed_since(t1) -> Option<u64>`    |
| `fx.install()` returns `()`        | `fx.install()?` returns `Result<()>`    |
| `MMP_CONSENT_VERSION`              | `CONSENT_PROTOCOL_VERSION`               |
| `resolve_endpoint() -> String`     | `resolve_endpoint() -> Cow<'static, str>`|

Compiler errors are mechanical: add `?` where a setter call previously
chained, and unwrap or match the `Option`/`Result` where a value was
previously returned bare.

---

## Verification

- All 14 audit items independently re-verified via static analysis pass.
- No `assert!`, `panic!`, `.unwrap()`, or `.expect()` reachable from the
  public API in production code paths (compile-time `const _: () =
  assert!(...)` width-guards excluded — these run at build time, not at
  runtime).
- Zero references to the legacy external protocol in source, Cargo
  metadata, or distributed documentation.

---

## Known limitations

- Real kernel transport is not yet wired. `IntentStream::try_next`
  requires either the `kernel-stub` feature (returns `Ok(None)`) or
  awaits L3 validation in Q2 2026 per RFC-0005.
- L3 oscilloscope-validated WCRT is **pending** Q2 2026. Until then,
  performance claims in this SDK are stated at L1 or L2 per
  RFC-0005.
- No same-hardware controlled benchmark against other RTOS platforms
  has been performed.

---

## Audit history

- **0.4.0 (May 2026):** Independent Rust security audit, 14 items
  resolved.
- **0.3.0 (April 2026):** Independent security audits (0.1.0–0.2.0)
  Critical and High findings resolved.

---

axonos.org · medium.com/@AxonOS · info@axonos.org
