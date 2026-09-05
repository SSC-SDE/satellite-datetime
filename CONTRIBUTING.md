# Contributing

This crate is **0.1 experimental**. Please read the README status section before sending a PR that claims flight or IAU completeness.

## License

Contributions are dual-licensed MIT OR Apache-2.0, the same as the crate.

## Checks

```bash
cargo fmt --all -- --check
cargo test --all-features
cargo test --no-default-features --lib
cargo clippy --all-features -- -D warnings
cargo publish --dry-run
```

MSRV is **1.85** (`package.rust-version` in `Cargo.toml`).

API docs appear on [docs.rs/satellite-datetime](https://docs.rs/satellite-datetime) only after `cargo publish`. Until then: `cargo doc --all-features --no-deps --open`.

## Science

Cite IAU/IERS/NASA sources in comments. Do not invent leap seconds, time zones, or a frozen LTC. Prefer golden vectors (ERFA, Mars24) over folklore constants.
