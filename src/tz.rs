//! IANA-compatible time zones with DST gaps and overlaps.
//!
//! Data pin: **2026a-subset**. Full tzdb is not embedded (flight-hostile). Named
//! zones use the post-2007 US Energy Policy Act rules, EU/UK last-Sunday rules,
//! and fixed offsets. Historical transitions before those laws are not applied.

use crate::earth::CivilUtc;
use crate::error::{Error, Result};
use crate::instant::Instant;
use crate::julian::unix_days_from_civil;

/// Pinned subset label (not a complete tzdb dump).
pub const TZDB_VERSION: &str = "2026a-subset-us2007-eu1996-fixed";

/// How a local civil time maps to UTC instants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocalTime {
    /// Unique mapping.
    Unique(Instant),
    /// Skipped at spring-forward. The civil time does not occur.
    Gap {
        /// Instant just before the gap (standard offset).
        before: Instant,
        /// Instant just after the gap (DST offset).
        after: Instant,
    },
    /// Repeated at fall-back. `first` is DST, `second` is standard (later Instant).
    Overlap {
        /// Earlier occurrence (DST).
        first: Instant,
        /// Later occurrence (standard time).
        second: Instant,
    },
}

/// UTC offset in force in a named zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneOffset {
    /// Seconds east of UTC (negative west).
    pub seconds_east: i32,
    /// Whether this offset is a daylight-saving offset in this zone's rules.
    pub is_dst: bool,
    /// Short abbreviation (e.g. `EST`).
    pub abbr: &'static str,
}

/// A named zone in the subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeZone {
    /// UTC (`Etc/UTC`).
    Utc,
    /// America/New_York (US DST 2007+).
    AmericaNewYork,
    /// America/Los_Angeles (US DST 2007+).
    AmericaLosAngeles,
    /// Europe/London (EU/UK last Sunday March/October).
    EuropeLondon,
    /// Europe/Paris (EU).
    EuropeParis,
    /// Asia/Kolkata (fixed +05:30, no DST).
    AsiaKolkata,
    /// Pacific/Auckland (NZ: last Sunday September / first Sunday April).
    PacificAuckland,
}

impl TimeZone {
    /// IANA-style identifier for this subset zone.
    pub fn iana_name(self) -> &'static str {
        match self {
            Self::Utc => "UTC",
            Self::AmericaNewYork => "America/New_York",
            Self::AmericaLosAngeles => "America/Los_Angeles",
            Self::EuropeLondon => "Europe/London",
            Self::EuropeParis => "Europe/Paris",
            Self::AsiaKolkata => "Asia/Kolkata",
            Self::PacificAuckland => "Pacific/Auckland",
        }
    }

    /// Look up a subset zone. Unknown names return [`Error::Unsupported`].
    pub fn from_iana(name: &str) -> Result<Self> {
        Ok(match name {
            "UTC" | "Etc/UTC" | "Zulu" => Self::Utc,
            "America/New_York" => Self::AmericaNewYork,
            "America/Los_Angeles" => Self::AmericaLosAngeles,
            "Europe/London" => Self::EuropeLondon,
            "Europe/Paris" => Self::EuropeParis,
            "Asia/Kolkata" | "Asia/Calcutta" => Self::AsiaKolkata,
            "Pacific/Auckland" => Self::PacificAuckland,
            _ => return Err(Error::Unsupported),
        })
    }

    /// Offset in force at `instant`.
    pub fn offset_at(self, instant: Instant) -> Result<ZoneOffset> {
        match self {
            Self::Utc => Ok(ZoneOffset {
                seconds_east: 0,
                is_dst: false,
                abbr: "UTC",
            }),
            Self::AsiaKolkata => Ok(ZoneOffset {
                seconds_east: 5 * 3600 + 1800,
                is_dst: false,
                abbr: "IST",
            }),
            Self::AmericaNewYork => us_offset(instant, -5 * 3600, "EST", "EDT"),
            Self::AmericaLosAngeles => us_offset(instant, -8 * 3600, "PST", "PDT"),
            Self::EuropeLondon => eu_offset(instant, 0, "GMT", "BST"),
            Self::EuropeParis => eu_offset(instant, 3600, "CET", "CEST"),
            Self::PacificAuckland => nz_offset(instant),
        }
    }

    /// Convert an instant to local civil clock (ISO fields, no zone suffix).
    pub fn to_local(self, instant: Instant) -> Result<(CivilUtc, ZoneOffset)> {
        let off = self.offset_at(instant)?;
        let shifted =
            instant.checked_add(crate::Duration::from_seconds(off.seconds_east as i64))?;
        let mut c = shifted.to_utc()?;
        // Local clock is not UTC; we reuse CivilUtc as a Y-M-D h:m:s container.
        if c.second == 60 {
            // Leap seconds are UTC-only; show 23:59:60 UTC then apply offset as 60s add.
            let s = instant.checked_add(crate::Duration::from_seconds(off.seconds_east as i64))?;
            c = s.to_utc()?;
        }
        Ok((c, off))
    }

    /// Interpret `local` as a wall clock in this zone.
    pub fn resolve(self, local: CivilUtc) -> Result<LocalTime> {
        if matches!(self, Self::Utc | Self::AsiaKolkata) {
            let off = self.offset_at(Instant::TAI_EPOCH)?;
            return Ok(LocalTime::Unique(local_to_instant(
                local,
                off.seconds_east,
            )?));
        }
        let std = std_seconds(self);
        let dst = dst_seconds(self);
        let as_std = local_to_instant(local, std)?;
        let as_dst = local_to_instant(local, dst)?;
        let off_std = self.offset_at(as_std)?;
        let off_dst = self.offset_at(as_dst)?;
        let std_ok = off_std.seconds_east == std;
        let dst_ok = off_dst.seconds_east == dst;
        match (std_ok, dst_ok) {
            (true, false) => Ok(LocalTime::Unique(as_std)),
            (false, true) => Ok(LocalTime::Unique(as_dst)),
            (true, true) => Ok(LocalTime::Overlap {
                first: as_dst,
                second: as_std,
            }),
            (false, false) => Ok(LocalTime::Gap {
                before: as_std,
                after: as_dst,
            }),
        }
    }
}

fn std_seconds(z: TimeZone) -> i32 {
    match z {
        TimeZone::AmericaNewYork => -5 * 3600,
        TimeZone::AmericaLosAngeles => -8 * 3600,
        TimeZone::EuropeLondon => 0,
        TimeZone::EuropeParis => 3600,
        TimeZone::PacificAuckland => 12 * 3600,
        TimeZone::Utc => 0,
        TimeZone::AsiaKolkata => 5 * 3600 + 1800,
    }
}

fn dst_seconds(z: TimeZone) -> i32 {
    match z {
        TimeZone::PacificAuckland => 13 * 3600,
        TimeZone::Utc | TimeZone::AsiaKolkata => std_seconds(z),
        _ => std_seconds(z) + 3600,
    }
}

fn local_to_instant(local: CivilUtc, offset_east: i32) -> Result<Instant> {
    // Treat fields as if they were UTC, then subtract the offset to get the Instant.
    let as_utc = local.to_instant()?;
    as_utc.checked_sub(crate::Duration::from_seconds(offset_east as i64))
}

fn us_offset(
    instant: Instant,
    std: i32,
    abbr_std: &'static str,
    abbr_dst: &'static str,
) -> Result<ZoneOffset> {
    let utc = instant.to_utc()?;
    let y = utc.year;
    // 02:00 local standard = 07:00 UTC for EST, 10:00 UTC for PST: utc = 02:00 - std/3600
    let spring = ymd_nth_weekday(y, 3, 0, 2); // 2nd Sunday March
    let fall = ymd_nth_weekday(y, 11, 0, 1); // 1st Sunday November
    let start_hour = ((2 * 3600 - std) / 3600) as u8;
    let end_hour = ((2 * 3600 - (std + 3600)) / 3600) as u8;
    let start = utc_hms(spring.0, spring.1, spring.2, start_hour)?;
    let end = utc_hms(fall.0, fall.1, fall.2, end_hour)?;
    let dst = instant >= start && instant < end;
    Ok(ZoneOffset {
        seconds_east: if dst { std + 3600 } else { std },
        is_dst: dst,
        abbr: if dst { abbr_dst } else { abbr_std },
    })
}

fn eu_offset(
    instant: Instant,
    std: i32,
    abbr_std: &'static str,
    abbr_dst: &'static str,
) -> Result<ZoneOffset> {
    let utc = instant.to_utc()?;
    let y = utc.year;
    let march = last_weekday(y, 3, 0);
    let oct = last_weekday(y, 10, 0);
    // 01:00 UTC start, 01:00 UTC end (EU).
    let start = utc_hms(march.0, march.1, march.2, 1)?;
    let end = utc_hms(oct.0, oct.1, oct.2, 1)?;
    let dst = instant >= start && instant < end;
    Ok(ZoneOffset {
        seconds_east: if dst { std + 3600 } else { std },
        is_dst: dst,
        abbr: if dst { abbr_dst } else { abbr_std },
    })
}

fn nz_offset(instant: Instant) -> Result<ZoneOffset> {
    let utc = instant.to_utc()?;
    let y = utc.year;
    // NZ: DST starts last Sunday of September 02:00 NZST (+12) = 14:00 UTC previous?
    // 02:00 NZST = 02:00-12h = 14:00 UTC of previous calendar day? 02:00 +12 means local 02:00 = UTC 14:00 previous day...
    // local = UTC + 12 ⇒ UTC = local - 12. 02:00 Sep last Sunday NZST → 14:00 UTC previous day if 02-12 negative.
    // 02:00 - 12h = 14:00 previous day UTC.
    let start_local = last_weekday(y, 9, 0);
    let end_local = ymd_nth_weekday(y + 1, 4, 0, 1);
    let start = CivilUtc::new(start_local.0, start_local.1, start_local.2, 2, 0, 0, 0)?
        .to_instant()?
        .checked_sub(crate::Duration::from_seconds(12 * 3600))?;
    let end = CivilUtc::new(end_local.0, end_local.1, end_local.2, 3, 0, 0, 0)?
        .to_instant()?
        .checked_sub(crate::Duration::from_seconds(13 * 3600))?;
    let dst = instant >= start && instant < end;
    Ok(ZoneOffset {
        seconds_east: if dst { 13 * 3600 } else { 12 * 3600 },
        is_dst: dst,
        abbr: if dst { "NZDT" } else { "NZST" },
    })
}

fn utc_hms(y: i32, m: u8, d: u8, hour: u8) -> Result<Instant> {
    CivilUtc::new(y, m, d, hour, 0, 0, 0)?.to_instant()
}

/// n-th weekday of month (0=Sunday), n>=1.
fn ymd_nth_weekday(year: i32, month: u8, weekday: u8, n: u8) -> (i32, u8, u8) {
    let first_wd = weekday_of(year, month, 1);
    let mut day = 1 + (weekday as i32 - first_wd as i32 + 7) % 7;
    day += (n as i32 - 1) * 7;
    (year, month, day as u8)
}

fn last_weekday(year: i32, month: u8, weekday: u8) -> (i32, u8, u8) {
    let dim = crate::earth::days_in_month(year, month).unwrap_or(30);
    let wd = weekday_of(year, month, dim);
    let delta = (wd as i32 - weekday as i32 + 7) % 7;
    (year, month, dim - delta as u8)
}

fn weekday_of(year: i32, month: u8, day: u8) -> u8 {
    // Unix day 0 = 1970-01-01 Thursday = 4
    let z = unix_days_from_civil(year, month, day);
    (((z + 4) % 7 + 7) % 7) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earth::CivilUtc;

    #[test]
    fn ny_spring_gap_2024() {
        let z = TimeZone::AmericaNewYork;
        let missing = CivilUtc::new(2024, 3, 10, 2, 30, 0, 0).unwrap();
        match z.resolve(missing).unwrap() {
            LocalTime::Gap { .. } => {}
            other => panic!("expected gap, got {other:?}"),
        }
        let ok = CivilUtc::new(2024, 3, 10, 3, 30, 0, 0).unwrap();
        match z.resolve(ok).unwrap() {
            LocalTime::Unique(_) => {}
            other => panic!("expected unique, got {other:?}"),
        }
    }

    #[test]
    fn ny_fall_overlap_2024() {
        let z = TimeZone::AmericaNewYork;
        let twice = CivilUtc::new(2024, 11, 3, 1, 30, 0, 0).unwrap();
        match z.resolve(twice).unwrap() {
            LocalTime::Overlap { first, second } => {
                assert!(second.duration_since(first).unwrap().as_seconds() == 3600);
            }
            other => panic!("expected overlap, got {other:?}"),
        }
    }

    #[test]
    fn kolkata_fixed() {
        let z = TimeZone::AsiaKolkata;
        let c = CivilUtc::new(2024, 6, 1, 12, 0, 0, 0).unwrap();
        match z.resolve(c).unwrap() {
            LocalTime::Unique(i) => {
                let utc = i.to_utc().unwrap();
                assert_eq!(utc.hour, 6);
                assert_eq!(utc.minute, 30);
            }
            _ => panic!("fixed offset"),
        }
    }
}
