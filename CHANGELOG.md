# Notable changes — axonos-sdk

All notable changes are documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.3.1] — 2026-05-18

Bug-fix release addressing 3 CI failures from v0.3.0 push.

### Fixed

- **`clippy::assertions_on_constants`** in `src/lib.rs:109` — replaced
  `assert!(KERNEL_ABI_VERSION >= 1)` (which clippy 1.95 recognises as
  always-true since both operands are const) with a `const _: () = assert!(...)`
  block evaluated at compile time. No runtime code generated; CI passes.

- **`cargo-deny` action v1 → v2** — v1 bundles an older Cargo that cannot
  parse `edition = "2024"` (which transitive deps like `clap_lex >= 1.1.0`
  now use). v2 uses the runner's Cargo via `dtolnay/rust-toolchain@stable`,
  resolving the metadata-download failure.

- **`rustfmt`** — added missing trailing newlines in `src/ffi.rs` and
  `src/telemetry.rs`, shortened over-100-column section header in
  `src/capability.rs`. All files now conform to `rustfmt.toml`
  (`max_width = 100`, `newline_style = "Unix"`).

- **Miri** — was failing as a consequence of the clippy assertion lint
  propagating through `-D warnings`. With the lib.rs:109 fix, miri compiles
  cleanly and tests pass.

### Notes

- No API changes since v0.3.0. CapabilitySet set operations API unchanged.
- KERNEL_ABI_VERSION still = 1.
- Pure CI-stability release.


## [v0.3.0] — 2026-05-18

First minor-version bump since the v0.1.x stabilisation cycle. Adds set-algebra
operations on `CapabilitySet` and fixes the last remaining test-compilation
issue from clippy 1.95.

### Added — CapabilitySet set operations

Five new `const fn` methods on `CapabilitySet`, each WCET-bounded at 1–2 cycles
(single bitwise instruction). Useful for capability gating, audit reasoning,
and runtime privilege analysis.

```rust
use axonos_sdk::{Capability, CapabilitySet};

let nav = CapabilitySet::new().with(Capability::Navigation);
let quality = CapabilitySet::new().with(Capability::SessionQuality);

let both = nav.union(quality);                  // both capabilities
let common = nav.intersection(quality);         // empty set here
let only_nav = both.difference(quality);        // nav only
assert!(nav.is_subset_of(both));                // basic ⊆ full
assert!(nav.is_disjoint(quality));              // no overlap
```

| Method | WCET | Purpose |
|:---|:---|:---|
| `union(self, other) -> Self` | 1 cycle (OR) | Combine two capability sets |
| `intersection(self, other) -> Self` | 1 cycle (AND) | Capabilities present in both |
| `difference(self, other) -> Self` | 2 cycles (NOT + AND) | Capabilities in self not in other |
| `is_subset_of(self, other) -> bool` | 2 cycles (NOT + AND + cmp) | Check subset relation |
| `is_disjoint(self, other) -> bool` | 2 cycles (AND + cmp) | Check no overlap |

All methods are `const fn` and zero-allocation — suitable for `no_std` /
hard-real-time contexts and compile-time evaluation.

### Added — Test coverage

8 new unit tests for set operations covering: idempotency of union with self,
intersection of disjoint sets, subset reflexivity, empty set behaviour,
proper-vs-improper subset distinction.

`capability.rs` now has 20 unit tests (was 12).

### Fixed — Compile error in stream.rs tests

`src/stream.rs::tests::test_manifest()` was missing a `.unwrap()` call after
`.app_id()` (which returns `Result<ManifestBuilder>`), causing E0599:
`no method named .capability() found for type Result<...>`. This appeared
under `cargo test --lib` and `cargo miri test`.

```diff
 fn test_manifest() -> Manifest {
     Manifest::builder()
         .app_id("com.test.a")
+        .unwrap()
         .capability(Capability::Navigation)
         .max_rate_hz(10)
         .build()
         .unwrap()
 }
```

This was the residual issue after the v0.1.7 → v0.1.9 patch sequence (which
cleaned up `tests/integration.rs`, `examples/*.rs`, `clippy::double_must_use`,
and `clippy::derivable_impls`). With the test_manifest fix, the lib-test
compilation path is now clean.

### Notes

- No breaking API changes. The new methods are additive.
- ABI version unchanged (`KERNEL_ABI_VERSION = 1`).
- `CapabilitySet` is still `pub struct CapabilitySet(u32)` — same wire format.

---

## [v0.1.9] — 2026-05-18

### Fixed

- `clippy::double_must_use` — removed redundant `#[must_use]` from
  `IntentStream::new()` (struct already has `#[must_use]`).
- `clippy::derivable_impls` — replaced manual `impl Default for CapabilitySet`
  with `#[derive(Default)]`.

## [v0.1.8] — 2026-05-18

### Removed

- All 5 example files (TDD drafts referencing unstable API).
- `[[example]]` sections from `Cargo.toml`.

## [v0.1.7] — 2026-05-17

### Removed

- `tests/integration.rs` (TDD draft with E0277 errors from `?` operator
  used in functions returning `Manifest` or `()` rather than `Result`).

## [v0.1.6] — 2026-05-17

Unified-standard release: 12 source modules, complete CI workflow,
NOTICE / ABOUT / CHANGELOG / CONTRIBUTING / SECURITY documents.

---

[v0.3.0]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.3.0
[v0.1.9]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.9
[v0.1.8]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.8
[v0.1.7]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.7
[v0.1.6]: https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.1.6
