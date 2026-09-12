# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This crate is **0.x**: entries may include breaking changes without a 1.0 bump.

## Unreleased

## [0.1.3] - 2026-09-13

### Added

- Hot-path Earth APIs on existing `CivilUtc`: `utc_mjd`, `dut1_at`, `julian_ut1_at`, `era_at_utc`, `gmst_mean_at_utc` (no `Instant::to_utc` search).
- `UtcDay` and `UtcContext` for O(1) same-day `Instant` → civil UTC in telemetry loops.
- Host Criterion benches in `benches/conversions.rs` (core, convenience UTC/DUT1/ERA, RFC 3339, CUC, plus `dut1_at_civil`, `era_at_utc_civil`, `utc_day_civil_from_instant`). Not a CI gate.

### Changed

- Documented the cost split: TAI core vs convenience `to_utc` vs hot-path `UtcDay` / `dut1_at`.
- Crate description mentions DUT1/ERA.

### Added

- Hot-path Earth APIs on existing [`CivilUtc`](https://docs.rs/satellite-datetime/latest/satellite_datetime/struct.CivilUtc.html): `utc_mjd`, `dut1_at`, `julian_ut1_at`, `era_at_utc`, `gmst_mean_at_utc` (no `Instant::to_utc` search).
- [`UtcDay`](https://docs.rs/satellite-datetime/latest/satellite_datetime/struct.UtcDay.html) and [`UtcContext`](https://docs.rs/satellite-datetime/latest/satellite_datetime/struct.UtcContext.html) for O(1) same-day `Instant` → civil UTC in telemetry loops.
- Criterion benches `dut1_at_civil`, `era_at_utc_civil`, `utc_day_civil_from_instant`.

## [0.1.2] - 2026-09-12

### Added

- Pinned IERS EOP C04 DUT1 (UT1−UTC) table on a 5-day knot grid from 1972-01-01 through MJD 61045.
- `earth::dut1`, `Instant::julian_ut1`, `Instant::earth_rotation_angle_rad` (IAU 2000 ERA), and `Instant::gmst_mean_rad` (IAU 2006 mean GMST).
- `scripts/gen_ut1_table.py` to regenerate `src/earth/ut1_table.rs` from IERS C04.
- Golden tests in `tests/iers_ut1_golden.rs`.

## [0.1.1] - 2026-09-09

### Added

- Open-source contribution pipeline: expanded CONTRIBUTING, Code of Conduct, issue/PR templates, `scripts/check.sh`.
- ERFA/SOFA cookbook golden tests for UTC↔TAI↔TT (IERS leap table, GPS epoch, civil round-trips).
- CI `cargo check --no-default-features` on `thumbv7em-none-eabihf` (Cortex-M4F satellite profile).

### Fixed

- UTC reconstruction at 1972-01-01 no longer treats the radio-era TAI−UTC step as a 23:59:60 leap second.
- `Duration::from_seconds_f64` uses `libm` rounding so the `no_std` core builds on `thumbv7em-none-eabihf`.

## [0.1.0] - 2026-09-06

### Added

- `no_std` `Instant` (TAI nanoseconds since 1958-01-01) and SI `Duration`.
- TAI, TT, TCG, TCB, TDB readings (TDB−TT is a two-term model).
- UTC leap-second table (IERS through 2016-12-31), RFC 3339, POSIX vs SI Unix.
- GNSS offsets (GPS, GST, BDT).
- IANA-compatible DST for a seven-zone subset (not full tzdb).
- IAU 2024 TCL origin and provisional LTC alias; mean lunar surface rate.
- NASA GISS MSD/MTC; IAU WGCCRE sidereal bodies; CCSDS CUC/CDS T-field.
- crates.io / [docs.rs](https://docs.rs/satellite-datetime) metadata (`documentation` URL). Hosted rustdoc appears only after `cargo publish`.

[0.1.3]: https://github.com/SSC-SDE/satellite-datetime/releases/tag/v0.1.3
[0.1.2]: https://github.com/SSC-SDE/satellite-datetime/releases/tag/v0.1.2
[0.1.1]: https://github.com/SSC-SDE/satellite-datetime/releases/tag/v0.1.1
[0.1.0]: https://github.com/SSC-SDE/satellite-datetime/releases/tag/v0.1.0
