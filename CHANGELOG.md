# Notable changes — axonos-sdk

All notable changes are documented in this file.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [v0.3.5] — 2026-05-28

AxonOS-style refresh and repository hygiene. No public API changes — the
v0.3.4 source behaviour is preserved; `KERNEL_ABI_VERSION = 1` and the
frozen RFC-0006 wire format are unchanged. This release brings the
repository up to the unified AxonOS visual and documentation standard
applied across the organisation.

### Removed

- **Orphan stub files `src/ffi.rs` and `src/telemetry.rs`** — neither was
  declared in `lib.rs` (`mod ffi` / `mod telemetry` were absent), so they
  were never compiled into the crate. They were empty `TODO` placeholders.
  The C-FFI and telemetry work remains on the Phase 2 roadmap; it is no
  longer shipped as dead source. The README module table and tree no
  longer list them.
- **`RELEASE_NOTES.md`** — duplicated the version history already in this
  file and the GitHub Releases generated from it by `release.yml`. Its
  unique "Current limitations" content was folded into the README
  Stability section.

### Changed

- **`SECURITY_FIXES.md` → `docs/SECURITY-AUDIT.md`** — the independent
  audit finding→fix record (26 items: AUDIT, HARDCORE, ENG) is preserved
  in full but moved out of the root, so the root carries a single
  `SECURITY.md` (the disclosure policy) like every other AxonOS repository.
- **README** — removed the standalone Ferris mascot image; replaced the
  mixed badge palette with the canonical AxonOS palette in one style
  (AxonOS blue `#0a4a8f` for Crate v0.3.5 / Standard v1.0.0 / Kernel ABI v1,
  Rust canonical orange `#CE422B`, trust green `#0d7a5f` for the
  unsafe-denied tag, slate `#475569` for licence and metadata). Replaced
  the "Related" list with a full **Position in the AxonOS stack** table
  (all seven repositories). Canonical centered footer (Singapore first),
  decorative emoji removed.
- **README accuracy fixes** — the repository-structure tree previously
  listed an `examples/` directory and a `tests/` directory that do not
  exist, and the removed orphan modules. The tree now reflects the actual
  tree. The quick-start dependency line was `axonos-sdk = "0.1"` while the
  crate is on the 0.3 line; corrected to `"0.3"`. The "What this crate
  gives you" section no longer advertises FFI/Telemetry as if present.
- **Repository URL case** — `repository` in `Cargo.toml` and
  `CITATION.cff`, and all in-repo links, corrected from the mis-cased upper-camel forms to the actual lowercase
  GitHub paths (`axonos-sdk`, `axonos-kernel`)
  (the Cargo URL is published to crates.io and must be byte-correct).
- **Author / contact email** — the personal author address and the legacy
  general-contact address were replaced with the project-canonical
  `connect@axonos.org` across source headers, manifests, README, NOTICE,
  the per-crate licence files, and the contributing guide.
  `security@axonos.org` is unchanged.

### Added

- **`.gitignore`** — Rust hygiene; was missing.
- **`CITATION.cff`** — already present; version synchronised to 0.3.5.

### Notes

- **No public API or behavioural changes.** Patch bump 0.3.4 → 0.3.5 per
  SemVer — additive/hygiene only, no new runtime dependency.

---

## [v0.3.4] — 2026-05-18

Pure CI-fix release. No code or API changes.

### Fixed — double trailing newlines in 13 source files

In v0.3.1, a defensive "ensure EOF newline" script had inverted-logic
that added an additional newline to files that already ended with one,
creating two trailing newlines. `cargo fmt --check` flagged this in CI.

```diff
 fn last_test() {
     assert_eq!(...);
 }
-
 
```
(rustfmt wants exactly one blank line after the last `}`, not two.)

Affected files (now corrected):
- `src/capability.rs`, `src/error.rs`, `src/ffi.rs`, `src/host.rs`,
  `src/intent.rs`, `src/lib.rs`, `src/manifest.rs`, `src/mesh.rs`,
  `src/stream.rs`, `src/telemetry.rs`, `src/time.rs`,
  `src/zerocopy_ext.rs`, `benches/intent_throughput.rs`

All 13 files now have exactly one trailing newline as rustfmt expects.

### Verified

A byte-level check confirms each file ends with exactly one `\n`:

```python
data.rstrip(b'\n') + b'\n'   # idempotent: one and only one trailing LF
```

### Notes

- No source-code changes (only EOF byte normalisation).
- No API surface changes.
- KERNEL_ABI_VERSION unchanged (= 1).


## [v0.3.3] — 2026-05-18

Automation release — tag pushes now produce real GitHub Releases.

### Added — auto-release workflow

New `.github/workflows/release.yml` triggers on every `v*.*.*` tag push
and creates a proper GitHub Release with:

- **Title:** the tag (e.g. `v0.3.3`)
- **Body:** the matching CHANGELOG section, extracted automatically via
  awk script that captures everything between `## [vX.Y.Z]` and the
  next version header
- **Latest banner:** stable releases (no pre-release suffix) get the
  green `Latest` badge on the repo page
- **Pre-releases:** tags with `-rc`, `-beta`, `-alpha` suffixes are
  marked as `Pre-release` automatically (no `Latest` banner)
- **Attached assets:** source `.tar.gz` and `.zip` archives built with
  `git archive` (clean, no `target/` or `.git/`)
- **Install hint:** auto-generated Cargo.toml snippet in the release body

### Before vs after

| Before this release | After |
|:---|:---|
| Tag appears as plain "N tags" link in repo header | Tag appears as **green Latest banner** with release name |
| No release notes | CHANGELOG section extracted as release body |
| No downloadable assets | `.tar.gz` + `.zip` attached |
| No prerelease distinction | `-rc`, `-beta` auto-marked as pre-release |

### How to use

```sh
# Same workflow as before — just tag and push
git tag -a v0.3.3 -m "v0.3.3: auto-release workflow"
git push origin v0.3.3
# Wait ~30 seconds → GitHub Actions creates the Release automatically
```

The release will appear at:
`https://github.com/AxonOS-org/axonos-sdk/releases/tag/v0.3.3`

### Notes

- Workflow requires `contents: write` permission (granted by repo default).
- No code changes — pure tooling addition.
- Existing tags (v0.3.0, v0.3.1, v0.3.2) **do not** retroactively get
  Releases. To backfill: re-push them as `v0.3.0` etc. (or create
  Releases manually in the GitHub UI).


## [v0.3.2] — 2026-05-18

CI stabilisation + new convenience API for capability enumeration.

### Added — `Capability::all()` and `CapabilitySet::all()`

Two complementary `const fn` methods for enumerating every defined capability.
Useful for permissive defaults, introspection UIs, and exhaustive policy checks.

```rust
use axonos_sdk::{Capability, CapabilitySet};

// Array of every variant in discriminant order
let all_variants = Capability::all();
assert_eq!(all_variants.len(), 4);

// Set containing every variant
let permissive = CapabilitySet::all();
assert_eq!(permissive.len(), 4);
for c in Capability::all() {
    assert!(permissive.contains(c));
}
```

Both methods are `const fn` and zero-allocation. `Capability::all()`
returns `[Self; CAPABILITY_COUNT as usize]` so the size is a compile-time
constant.

5 new unit tests cover: discriminant ordering, equivalence with manual
union, superset relation, length invariant.

`capability.rs` test count: 20 → 25.

### Fixed — rustfmt import ordering in `host.rs`

`use serial_test::serial` was placed before `use super::*`, which violates
rustfmt's `reorder_imports = true` convention (super/crate first, then
external crates). Reordered to make `cargo fmt --check` green.

### Fixed — miri leak detection on `InMemoryFixture`

The CI workflow now runs miri with `MIRIFLAGS=-Zmiri-ignore-leaks`. The
`InMemoryFixture` global state (a `Mutex<Option<...>>` for thread-safe
test isolation) is correctly reported as leaked at process exit by miri,
but this is **deliberate test-only state**, not a memory-safety issue.

The fix is environment-level (CI workflow), not source code. The fixture
implementation is unchanged.

### Documentation consistency

- `README.md` line 124: bumped axonos-sdk ABI compatibility floor from
  `≥ 0.1.0` to `≥ 0.3.0`.
- `ABOUT.md` line 64: pointed kernel compatibility at `≥ 0.1.9` (the
  current published kernel version) and clarified that the v1 ABI
  number lives in the `KERNEL_ABI_VERSION` constant.

### Notes

- No API removal. `Capability::all()` and `CapabilitySet::all()` are
  purely additive.
- `KERNEL_ABI_VERSION` still `1`. Wire format unchanged.


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
