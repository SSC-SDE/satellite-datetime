# satellite-datetime

[![crates.io](https://img.shields.io/crates/v/satellite-datetime.svg)](https://crates.io/crates/satellite-datetime)
[![docs.rs](https://img.shields.io/docsrs/satellite-datetime)](https://docs.rs/satellite-datetime)
[![License](https://img.shields.io/crates/l/satellite-datetime.svg)](LICENSE-MIT)

**0.1 work-in-progress.** APIs will break. This is **not** flight-qualified software.
Time zones are a **seven-zone subset**, not the IANA database. Lunar TCL has the IAU
2024 origin but **no periodic BCRS series**. LTC is a placeholder until BIPM/CGPM
publish an operational offset.

Rust timescales for spacecraft, the Moon, and the solar system. An **instant is not a calendar**: the core stores TAI nanoseconds since 1958-01-01. UTC leap seconds, DST, lunar TCL, and Mars sols are projections of that instant.

## Install

```toml
[dependencies]
satellite-datetime = "0.1"
```

Satellite / `no_std` (no default features, no allocator):

```toml
[dependencies]
satellite-datetime = { version = "0.1", default-features = false }
```

MSRV: **1.85**. License: **MIT OR Apache-2.0**. SemVer: `0.x` may break in any minor or patch as the science and API settle; pin an exact version in production experiments.

## Documentation

API reference: **[docs.rs/satellite-datetime](https://docs.rs/satellite-datetime)**.

Generate rustdoc locally:

```bash
cargo doc --all-features --no-deps --open
```

## Satellite profile

Core types compile without an OS or heap:

```bash
cargo test --no-default-features --lib
```

A laptop build (`cargo test`) enables Earth civil time, time zones, GNSS, Moon, Mars, generic bodies, and CCSDS codecs.

There is no `now()` in the core. Inject a clock.

## Timescales

| Scale | Role |
| --- | --- |
| **TAI** | Continuous SI seconds. Internal representation. |
| **TT** | TAI + 32.184 s (exact). Ephemeris lookup on Earth. |
| **TCG / TCB / TDB** | IAU relativistic coordinate times (`L_G`, `L_B`, few-term TDB−TT). |
| **UTC** | TAI minus IERS leap seconds (table pinned through 2016-12-31). |
| **POSIX Unix** | Civil seconds **without** leaps; `si_nanos_since_unix_epoch` counts real SI. |
| **GPS / GST / BDT** | Fixed offsets from TAI. |
| **TCL** | IAU 2024 Lunar Coordinate Time (origin 1977 with TCB). Periodic BCRS terms omitted. |
| **LTC** | Provisional: currently identical to TCL until BIPM/CGPM freeze an operational offset. |
| **MSD / MTC** | NASA GISS / Allison & McEwen Mars sol time (UT1 analog, not leap UTC). |

Relativity: converting **coordinate** times does not need a trajectory. Converting to **proper time of a clock** does. Lunar surface proper time uses the published mean rate +56.02 µs/day vs TT (Ashby & Patla class result), not a claim of nanosecond geolocation.

## Features

| Feature | Default | Contents |
| --- | --- | --- |
| *(none)* | | `Instant`, `Duration`, TAI, TT, TCG, TCB, TDB |
| `earth` | yes | Gregorian, UTC leaps, ISO 8601 / RFC 3339, POSIX Unix |
| `tz` | yes | IANA subset `2026a-subset` (NY, LA, London, Paris, Kolkata, Auckland, UTC) |
| `gnss` | yes | GPS week/SoW, Galileo, BeiDou |
| `lunar` | yes | TCL, provisional LTC, mean lunar surface proper |
| `mars` | yes | MSD, MTC |
| `bodies` | yes | IAU WGCCRE sidereal prime meridian, Mercury–Neptune |
| `ccsds` | yes | CCSDS 301.0-B-4 CUC and CDS |
| `std` / `alloc` | yes | `std::error::Error`; formatting still works on `no_std` via buffers |

Leap seconds, UT1, and tz data are **pinned tables**, never fetched on-device.

## Example

```rust
use satellite_datetime::{parse_rfc3339, Instant};

let t: Instant = parse_rfc3339("2010-07-24T11:18:07.318Z").unwrap();
assert_eq!(t.to_utc().unwrap().second, 7);
```

Leap second:

```rust
use satellite_datetime::parse_rfc3339;
let leap = parse_rfc3339("2016-12-31T23:59:60Z").unwrap();
assert_eq!(leap.to_utc().unwrap().second, 60);
```

## Accuracy notes

- UTC↔TAI after 1972: integer leap seconds from IERS Bulletin C (numeric table).
- Pre-1972 UTC: IERS `tai-utc.dat` drift terms (same numbers SOFA/ERFA use).
- TDB−TT: two-term annual model (~1.6 ms); not ERFA `dtdb` (needs site).
- TCL: origin-correct; linear TCB identification without lunar periodic series.
- `f64` Julian dates are ~50 µs near J2000; instants stay `i128` nanoseconds.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the fork → PR workflow and
[`./scripts/check.sh`](scripts/check.sh) for local CI checks. Please follow the
[Code of Conduct](CODE_OF_CONDUCT.md). Pull requests run GitHub Actions automatically.
