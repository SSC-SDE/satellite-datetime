# satellite-datetime

Rust timescales for spacecraft, the Moon, and the solar system. An **instant is not a calendar**: the core stores TAI nanoseconds since 1958-01-01. UTC leap seconds, IANA DST, lunar TCL, and Mars sols are projections of that instant.

This repository is **private** until it is explicitly open-sourced.

## Satellite profile

Flight software often has no OS and no heap. Core types compile with:

```bash
cargo test --no-default-features
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

MIT OR Apache-2.0.
