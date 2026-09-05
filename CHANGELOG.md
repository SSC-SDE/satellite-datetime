# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This crate is **0.x**: entries may include breaking changes without a 1.0 bump.

## [0.1.0] - 2026-09-06

### Added

- `no_std` `Instant` (TAI nanoseconds since 1958-01-01) and SI `Duration`.
- TAI, TT, TCG, TCB, TDB readings (TDB−TT is a two-term model).
- UTC leap-second table (IERS through 2016-12-31), RFC 3339, POSIX vs SI Unix.
- GNSS offsets (GPS, GST, BDT).
- IANA-compatible DST for a seven-zone subset (not full tzdb).
- IAU 2024 TCL origin and provisional LTC alias; mean lunar surface rate.
- NASA GISS MSD/MTC; IAU WGCCRE sidereal bodies; CCSDS CUC/CDS T-field.

[0.1.0]: https://github.com/SSC-SDE/satellite-datetime/releases/tag/v0.1.0
