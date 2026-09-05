# Contributing to satellite-datetime

Thank you for helping improve this crate. **satellite-datetime** is **0.1 work-in-progress**:
APIs may break, and it is **not** flight-qualified. Please read the [README](README.md) status
section before claiming IAU completeness, full tzdb coverage, or operational lunar time.

By contributing, you agree that your work is licensed under **MIT OR Apache-2.0**, the same as
this project. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community expectations.

## Quick start

1. **Fork** [SSC-SDE/satellite-datetime](https://github.com/SSC-SDE/satellite-datetime) on GitHub.
2. **Clone** your fork and create a branch:
   ```bash
   git checkout -b feat/short-description
   ```
3. **Change** code and/or tests. Keep diffs focused.
4. **Run checks** locally (same commands as CI):
   ```bash
   ./scripts/check.sh
   ```
5. **Commit** with a clear message (what and why).
6. **Push** to your fork and open a **pull request** against `main`.

Every PR runs [GitHub Actions CI](.github/workflows/ci.yml) automatically (format, tests,
clippy, package, MSRV).

## What we look for in PRs

- **Tests** for behavior changes (leap seconds, conversions, codecs, DST gaps/overlaps, etc.).
- **Small scope** — one logical change per PR when possible.
- **Changelog** — add a bullet under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md) for
  user-visible changes (new API, fixes, breaking changes).
- **Science** — cite IAU / IERS / NASA / CCSDS sources in comments or PR description. Prefer
  golden vectors (ERFA, Mars24) over folklore constants.
- **Honesty** — document uncertainty and limitations in rustdoc when accuracy is approximate.

## Scope guardrails

Please **do not** (without maintainer discussion):

- Invent leap seconds, Martian DST, or a frozen **LTC** before BIPM/CGPM publish one.
- Add `Instant::now()` or host-OS timezone lookups in the `no_std` core.
- Expand the `tz` feature with hand-rolled rules instead of a tzdb plan (full IANA is a
  separate milestone).
- Copy SOFA/ERFA source — reimplement published formulas and test against vectors.

## Local development

**Requirements:** Rust **1.85+** (`rust-version` in `Cargo.toml`).

```bash
# Full laptop profile (default features)
cargo test --all-features

# Satellite / no_std core only
cargo test --no-default-features --lib

# API docs (also on https://docs.rs/satellite-datetime)
cargo doc --all-features --no-deps --open
```

### One-command check

```bash
./scripts/check.sh
```

This runs: `cargo fmt --check`, both test profiles, `clippy -D warnings`, and
`cargo package --locked`.

## Reporting issues

- **Bugs and features:** use GitHub [Issues](https://github.com/SSC-SDE/satellite-datetime/issues)
  (templates provided).
- **Security** (conversion bugs that could affect navigation): use
  [Security advisories](https://github.com/SSC-SDE/satellite-datetime/security/advisories/new),
  not a public issue. See [SECURITY.md](SECURITY.md).

## Questions

Open a GitHub issue with the **question** label or start a discussion if enabled on the repo.
For crate usage, see [docs.rs](https://docs.rs/satellite-datetime).
