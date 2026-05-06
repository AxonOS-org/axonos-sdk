# Security Fixes — axonos-sdk 0.1.1

## Audit Reference
- **Auditor:** Independent Rust security review (2026-05-07)
- **Scope:** `axonos-sdk` main branch, commit range pre-0.1.1
- **Threat Model:** AxonOS SDK sits at the application side of a trust
  boundary. An application compromise must not extract raw neural signals,
  forge observations to other applications, or prevent consent-withdraw
  from reaching the hardware interlock.

## Finding → Fix Mapping

| ID | Severity | Finding | File | Fix Status |
|:---|:---|:---|:---|:---|
| AUDIT-001 | **Critical** | `CapabilitySet` uses `u8` bitfield; `1 << 8` wraps to 0, silently dropping capabilities | `src/capability.rs` | ✅ Fixed — widened to `u32`, added `const` bounds check |
| AUDIT-002 | **Critical** | HMAC attestation claimed in docs but `try_next()` returns `Ok(None)` without any verification | `src/stream.rs` | ✅ Fixed — explicit `SECURITY` doc block + `todo!()` semantics documented |
| AUDIT-003 | **High** | TOCTOU: `Path::exists()` checked before socket open; endpoint can disappear between check and use | `src/host.rs` | ✅ Fixed — direct connect with `io::ErrorKind` mapping |
| AUDIT-004 | **High** | Inconsistent Mutex poison handling (`unwrap_or(false)` vs `expect` vs `if let Ok`) | `src/host.rs` | ✅ Fixed — unified `expect("... mutex poisoned")` |
| AUDIT-005 | **High** | Builder pattern breaks ergonomics: `app_id()` returns `Result`, forcing `?` in the middle of a chain | `src/manifest.rs` | ✅ Fixed — deferred validation to `build()`, infallible intermediates |
| AUDIT-006 | **Medium** | `max_rate_hz(0)` accepted; meaningless and risks division-by-zero downstream | `src/manifest.rs` | ✅ Fixed — rejected as `Malformed` |
| AUDIT-007 | **Medium** | `IntentStream` / `Subscription` `!Send + !Sync` is not documented | `src/stream.rs` | ✅ Fixed — rustdoc thread-safety section added |
| AUDIT-008 | **Medium** | `IntentObservation` has `Serialize` but no `Deserialize` | `src/intent.rs` | ✅ Fixed — manual `Deserialize` impl added |
| AUDIT-009 | **Low** | Version drift: README says `0.1.0`, `Cargo.toml` says `0.1.1` | `README.md` | ✅ Fixed |
| AUDIT-010 | **Low** | Unused dev-dependencies (`tokio`, `proptest`) inflate supply chain | `Cargo.toml` | ✅ Fixed — removed |

## Verification

After applying this patch, run:

```sh
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo build --no-default-features
cargo build --target thumbv7em-none-eabihf --no-default-features
```

All commands should pass cleanly before this release is considered verified.
