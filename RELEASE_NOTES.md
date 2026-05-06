# Release Notes — axonos-sdk 0.1.1

## Security & Correctness Patch

This release addresses all Critical and High severity findings from the
independent security audit (2026-05-07). Downstream medical-device and
BCI integrators are strongly encouraged to upgrade before deploying to
any pre-clinical or clinical environment.

### Critical

- **CapabilitySet bitfield widened to `u32`** (`src/capability.rs`)
  - Previously `u8`, which would silently wrap on `1 << 8` if a future
    `Capability` variant with discriminant ≥ 8 were added. This is now
    a compile-time breaking change (the `const` assertion fails), preventing
    silent capability corruption and potential privilege escalation.
  - Added compile-time guard: `assert!((max_discriminant as usize) < 32)`.

- **Explicit HMAC attestation stub marker** (`src/stream.rs`)
  - `try_next()` now carries a detailed `SECURITY` comment explaining that
    truncated HMAC-SHA256 verification is pending kernel ABI stabilization.
  - Prevents false sense of security for auditors reviewing the crate.

### High

- **Eliminated TOCTOU in endpoint discovery** (`src/host.rs`)
  - Removed `Path::exists()` check before socket open. The transport now
    attempts the connection directly and maps the resulting `io::ErrorKind`
    to the appropriate `TransportFault`. This closes the race between check
    and use on Unix domain sockets and Windows named pipes.

- **Unified Mutex poison handling** (`src/host.rs`)
  - All `Mutex` lock sites now use `expect("... mutex poisoned")` with
    consistent, descriptive messages. Previously `fixture_installed()`
    silently returned `false` on poison, leading to non-deterministic test
    failures.

- **ManifestBuilder: infallible intermediate steps** (`src/manifest.rs`)
  - `app_id()`, `name()`, `vendor()` now return `Self` instead of
    `Result<Self>`. Validation is deferred to `build()`, enabling ergonomic
    chained construction without interleaving `?` operators.
  - Added `MAX_DISPLAY_STRING_LEN` constant (64 bytes) for name/vendor
    limits, validated in `build()`.

- **`max_rate_hz(0)` rejected** (`src/manifest.rs`)
  - A rate of `0 Hz` is now rejected as `Malformed`, preventing potential
    division-by-zero in downstream rate-limiting logic.

### Medium

- **Added `serde::Deserialize` for `IntentObservation`** (`src/intent.rs`)
  - Enables symmetric JSON/CBOR wire format: the SDK can now both serialize
    and deserialize its own observations.

- **Documented `!Send + !Sync` contract** (`src/stream.rs`)
  - Added explicit rustdoc notes explaining that `IntentStream` and
    `Subscription` are intentionally `!Send + !Sync` due to kernel IPC
    thread-affinity requirements.

- **Removed unused dev-dependencies** (`Cargo.toml`)
  - Dropped `tokio` and `proptest` (not used in any test or example).
  - Reduces compile times and supply-chain attack surface.

- **Fixed `bare_metal_no_std` example feature gate**
  - `required-features` changed from `["std"]` to `[]`, allowing the example
    to be built with `cargo build --example bare_metal_no_std` without
    forcing `std`.

### Low / Quality

- README version string synchronized with `Cargo.toml` (`0.1.1`).
- Added integration test for `CapabilitySet` u32 width exhaustiveness.
- Added integration test for infallible builder intermediate steps.

---

**Full audit report:** See `SECURITY_FIXES.md` for the original finding
identifiers and remediation mapping.
