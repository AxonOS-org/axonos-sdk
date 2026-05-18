# Security audit notes — axonos-sdk

## Audit Reference
- **Auditors:** Independent Rust security review (2026-05-07) + embedded/BCI engineer review
- **Scope:** Full source tree as of audit date
- **Threat Model:** AxonOS SDK at application/kernel boundary. Compromise must not extract raw EEG, forge observations, or prevent consent-withdraw from reaching hardware interlock.

## Finding → Fix Mapping

| ID | Severity | Finding | File | Fix |
|:---|:---|:---|:---|:---|
| AUDIT-001 | Critical | `CapabilitySet` u8 wrap | `capability.rs` | `u32` + `const` bounds |
| AUDIT-002 | Critical | HMAC attestation claimed but unimplemented | `stream.rs` | `compile_error!` without `kernel-stub` |
| AUDIT-003 | High | TOCTOU in endpoint discovery | `host.rs` | Unconditional `EndpointNotFound` |
| AUDIT-004 | High | Inconsistent mutex poison | `host.rs` | `into_inner()` recovery, no `expect()` |
| AUDIT-005 | High | Builder returns `Result` mid-chain | `manifest.rs` | Infallible intermediates |
| AUDIT-006 | Medium | `max_rate_hz(0)` accepted | `manifest.rs` | Rejected as `Malformed` |
| AUDIT-007 | Medium | `!Send+!Sync` undocumented | `stream.rs` | rustdoc section |
| AUDIT-008 | Medium | Missing `Deserialize` | `intent.rs` | Manual impl added |
| AUDIT-009 | Low | Version drift | `README.md` | Synchronized |
| AUDIT-010 | Low | Unused dev-deps | `Cargo.toml` | Removed |
| HARDCORE-001 | Critical | `map_endpoint_error` blocking hang | `host.rs` | Removed entirely |
| HARDCORE-002 | Critical | `align_of` fails on 32-bit | `intent.rs` | `#[repr(C, align(8))]` + target-gate |
| HARDCORE-003 | Critical | `f32` confidence non-determinism | `intent.rs` | Q0.16 fixed-point |
| HARDCORE-004 | High | Generic `PhantomData` | `stream.rs` | `PhantomData<SubscriptionInner>` |
| HARDCORE-005 | High | `as_raw()` leaks internals | `capability.rs` | `RawCapabilitySet` opaque wrapper |
| HARDCORE-006 | High | Silent truncation | `manifest.rs` | `assert!()` + `panic!()` |
| HARDCORE-007 | High | `uninstall()` ignores poison | `host.rs` | `into_inner()` recovery |
| HARDCORE-008 | Medium | `WithdrawReason` string serde | `mesh.rs` | `serde_repr` |
| HARDCORE-009 | Medium | Missing `siphasher` dep | `Cargo.toml` | Inline FNV-1a |
| HARDCORE-010 | Low | `try_next()` docs mismatch | `stream.rs` | `compile_error!` |
| ENG-001 | Critical | `unimplemented!()` in security path | `stream.rs` | `compile_error!` |
| ENG-002 | High | `expect()` in sync primitives | `host.rs` | Poison recovery |
| ENG-003 | High | `MeshClient` fake readiness | `mesh.rs` | `MeshClientStub` |
| ENG-004 | Medium | No time model | `time.rs` | `MonotonicTimestamp` + WCET |
| ENG-005 | Medium | No fixed-point spec | `intent.rs` | Q0.16 documented |
| ENG-006 | Low | `forbid(unsafe_code)` too rigid | `lib.rs` | `deny` + audited module |

## Verification

```sh
# Development build (with stub)
cargo test --all-features --features kernel-stub
cargo clippy --all-features --features kernel-stub -- -D warnings
cargo fmt --check

# Production build (must fail without kernel)
cargo build --no-default-features  # expected: compile_error in stream.rs

# Embedded target
cargo build --target thumbv7em-none-eabihf --no-default-features --features kernel-stub
```

All commands must pass (or fail as expected) before release.
