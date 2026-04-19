# Contributing to axonos-sdk

Thank you for your interest in AxonOS. This document describes how to contribute code, documentation, and issue reports.

## Before you start

- **Read the Medium research series** at https://medium.com/@AxonOS to understand the project's design philosophy. AxonOS is a safety-critical BCI infrastructure project; contributions are evaluated against that standard.
- **For protocol changes**, coordinate with the SYM.BOT maintainers of the MMP base protocol at https://sym.bot/spec/mmp. Consent-extension changes must be co-signed.

## Quick start

```sh
git clone https://github.com/AxonOS-org/axonos-sdk
cd axonos-sdk

# Run the test suite (all features).
cargo test --all-features

# Run the lints we enforce in CI.
cargo clippy --all-features -- -D warnings

# Check formatting.
cargo fmt --check

# Verify no_std builds.
cargo build --no-default-features
cargo build --example bare_metal_no_std --no-default-features
```

All four commands must pass before a PR will be reviewed.

## Guidelines

### Scope

This repository is the **public SDK** — the application-facing surface. It deliberately does not contain:

- Signal processing algorithms (that lives in the private `axonos-kernel` repository).
- Classifier weights or models.
- Hardware drivers beyond the HAL boundary.

Pull requests adding any of the above will be closed with a pointer to the appropriate repository.

### Code style

- `#![forbid(unsafe_code)]` is not negotiable. We do not accept PRs that introduce `unsafe` blocks to this crate.
- Every public item must have a doc comment with at least one sentence. Non-trivial items should have `# Example` and `# Errors` sections where applicable.
- Use `#[must_use]` on constructors and builders.
- Prefer `const fn` where possible.
- Prefer `#[non_exhaustive]` on public enums.
- Use `heapless` collections in the public API; never `Vec` or `HashMap` for data that might be constructed on the embedded path.

### Tests

- Every public function gets at least one unit test.
- Integration tests go in `tests/`. Use `InMemoryFixture` rather than mocking the transport.
- Benchmarks go in `benches/`. Use `criterion` and `black_box` correctly.
- If your PR changes behavior, add a test that fails without the change and passes with it. Link the failing test in the PR description.

### Changelog

Every PR that changes public API must update `CHANGELOG.md` under the `[Unreleased]` heading.

### Licensing

By submitting a PR, you agree your contribution is dual-licensed under Apache-2.0 and MIT (the crate's dual license). If you cannot agree to this, do not submit.

## Issue reports

- **Bugs:** include a minimal reproducer, the Rust version (`rustc --version`), the feature flags you used, and the full error output.
- **Feature requests:** state the problem you're solving, not the solution you want. We will discuss approaches.
- **Security issues:** **do not open a public issue.** See `SECURITY.md`.

## Code of Conduct

Be kind. Be technically honest. Disagree on merit, not identity. The project reserves the right to restrict access for repeated bad-faith interactions.

## Questions

General questions → GitHub Discussions or the Medium comment threads.
Commercial/enterprise questions → `axonosorg@gmail.com`.

---

`axonos.org · medium.com/@AxonOS · axonosorg@gmail.com`
