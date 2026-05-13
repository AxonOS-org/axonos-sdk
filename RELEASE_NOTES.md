# Release Notes — axonos-sdk 0.3.0

## Production-Hardened BCI SDK

This release addresses all Critical and High severity findings from independent security audits (0.1.0–0.2.0), plus architectural feedback from embedded/BCI engineers.

### Critical

- **`try_next()` → `compile_error!`** (`src/stream.rs`)
  - Without `kernel-stub` feature, the build fails at compile time with a clear message.
  - No runtime panic. No silent no-op. No false sense of security.
  - `kernel-stub` is explicitly marked as **development-only** in Cargo.toml and docs.

- **Fixed-point Q0.16 confidence** (`src/intent.rs`)
  - `u16` where `65535 == 1.0`. Deterministic across x86_64 SSE, Cortex-M4F FPU, soft-float.
  - `confidence_f32()` is **display-only**, documented as non-deterministic.

- **Portable layout** (`src/intent.rs`)
  - `#[repr(C, align(8))]` with target-gated size assertions.
  - Compiles on `thumbv7em-none-eabihf` (verified).

### High

- **Mutex poison recovery** (`src/host.rs`)
  - All `Mutex::lock()` sites use `match` + `into_inner()` recovery.
  - No `expect()`. No panic in sync primitives. Logs condition via `eprintln!` in debug.

- **Explicit stub naming** (`src/mesh.rs`)
  - `MeshClientStub` replaces `MeshClient`. No false impression of readiness.
  - Comprehensive rustdoc warning: "This is not a real client."

- **Opaque `RawCapabilitySet`** (`src/capability.rs`)
  - Internal bit layout hidden. `has_reserved_bits()` for wire validation.
  - `Display` + `audit_format()` for deterministic logging.

- **No silent truncation** (`src/manifest.rs`)
  - `assert!()` + `panic!()` on overflow. Infallible builder intermediates.

### Medium

- **`MonotonicTimestamp`** (`src/time.rs`)
  - New module with WCET documentation per operation.
  - `checked_sub()` + defensive `elapsed_since()`.
  - `#[repr(transparent)]` u64.

- **`#[deny(unsafe_code)]` with audited module** (`src/lib.rs`)
  - Global `deny`. Single `zerocopy_ext` module with `allow(unsafe_code)`.
  - Empty placeholder with safety contract documentation.

- **`serde_repr` on `WithdrawReason`** (`src/mesh.rs`)
  - Numeric `u8` wire format per AxonOS Consent Protocol spec.

### Low

- README badges with real hyperlinks.
- Email `info@axonos.org`.
- `bare_metal_no_std` requires `kernel-stub` (explicit opt-in).
- Examples updated for `MonotonicTimestamp` API.

---

**Full audit reports:** See `SECURITY_FIXES.md`.
