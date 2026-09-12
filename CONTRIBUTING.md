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

Every PR runs the staged [GitHub Actions pipeline](.github/workflows/pipeline.yml):
**dev** (fmt + tests) → **qa** (clippy, package, MSRV, embedded). Merges to `main` also run
**pre-prod** (docs, publish dry-run). Tag `v*` on `main` runs **prod** (crates.io publish).

## What we look for in PRs

- **Tests** for behavior changes (leap seconds, conversions, codecs, DST gaps/overlaps, etc.).
- **Small scope** — one logical change per PR when possible.
- **Changelog** — add a bullet under `## Unreleased` in [CHANGELOG.md](CHANGELOG.md) for
  user-visible changes (new API, fixes, breaking changes).
- **Science** — cite IAU / IERS / NASA / CCSDS sources in comments or PR description. Prefer
  golden vectors (ERFA, Mars24) over folklore constants. UTC↔TAI↔TT pairs live in
  [`tests/erfa_golden.rs`](tests/erfa_golden.rs); DUT1/ERA pairs in
  [`tests/iers_ut1_golden.rs`](tests/iers_ut1_golden.rs).
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

# Optional: host conversion benchmarks (not part of CI or ./scripts/check.sh)
cargo bench --bench conversions
```

### One-command check

```bash
./scripts/check.sh
```

This runs the **dev** and **qa** stages locally: `cargo fmt --check`, both test profiles,
`clippy -D warnings`, `cargo package --locked`, and `thumbv7em` when the target is installed.

**Benchmarks** (`cargo bench --bench conversions`) are optional and host-only. They are not run
in CI and must not be used as flight-qualification or science-accuracy metrics.

## CI/CD stages (main only)

All work lands on `main`. There are no long-lived `dev`/`qa` branches — stages are pipeline jobs:

| Stage | When | What |
| --- | --- | --- |
| **dev** | Every PR and push | `fmt`, unit tests (all features + `no_std` lib) |
| **qa** | After dev passes | `clippy`, example, `cargo package`, MSRV 1.85, `thumbv7em` check |
| **pre-prod** | Push to `main` only | Lockfile check, `cargo doc`, `cargo publish --dry-run` |
| **prod** | Push tag `v*` on `main` | `cargo publish` to crates.io (`CARGO_REGISTRY_TOKEN` in **prod** environment) |

Configure optional approval gates in GitHub → **Settings → Environments** (`dev`, `qa`, `pre-prod`, `prod`).

**Release flow:** merge to `main` → pre-prod green → tag `v0.1.3` → prod publishes.

## Reporting issues

- **Bugs and features:** use GitHub [Issues](https://github.com/SSC-SDE/satellite-datetime/issues)
  (templates provided).
- **Security** (conversion bugs that could affect navigation): use
  [Security advisories](https://github.com/SSC-SDE/satellite-datetime/security/advisories/new),
  not a public issue. See [SECURITY.md](SECURITY.md).

## Questions

Open a GitHub issue with the **question** label or start a discussion if enabled on the repo.
For crate usage, see [docs.rs](https://docs.rs/satellite-datetime).
