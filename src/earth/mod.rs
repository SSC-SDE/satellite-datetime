//! Earth civil time: leap seconds, Gregorian, ISO 8601, POSIX Unix.

mod iso8601;
mod leap;
mod unix;

pub use iso8601::{format_rfc3339, parse_rfc3339};
pub use leap::{leap_seconds_on_utc_day, tai_minus_utc, LeapInfo};
pub use unix::{from_posix_nanos, from_posix_seconds, si_nanos_since_unix_epoch, to_posix_seconds};

use crate::constants::{NS_PER_DAY, NS_PER_SEC, TAI_EPOCH_UNIX_DAYS};
use crate::duration::Duration;
use crate::error::{Error, Result};
use crate::instant::Instant;
use crate::julian::{civil_from_unix_days, unix_days_from_civil};

/// Proleptic Gregorian civil date and clock, tagged as UTC (may include second=60).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CivilUtc {
    /// Proleptic Gregorian year.
    pub year: i32,
    /// Month `1..=12`.
    pub month: u8,
    /// Day of month, `1..=31` (validated).
    pub day: u8,
    /// Hour `0..=23`.
    pub hour: u8,
    /// Minute `0..=59`.
    pub minute: u8,
    /// Second `0..=60` (`60` only on a positive UTC leap second).
    pub second: u8,
    /// Nanosecond `0..=999_999_999`.
    pub nanosecond: u32,
}

impl CivilUtc {
    /// Validate ranges. `second` may be 60 (leap). Month 1–12, etc.
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self> {
        if !(1..=12).contains(&month)
            || hour > 23
            || minute > 59
            || second > 60
            || nanosecond > 999_999_999
        {
            return Err(Error::InvalidTime);
        }
        if second == 60 && (hour != 23 || minute != 59) {
            return Err(Error::InvalidTime);
        }
        let dim = days_in_month(year, month)?;
        if day < 1 || day > dim {
            return Err(Error::InvalidDate);
        }
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            nanosecond,
        })
    }

    /// Convert this UTC civil time to a TAI instant using the pinned leap table.
    pub fn to_instant(self) -> Result<Instant> {
        if self.year < 1960 {
            return Err(Error::UtcUndefined);
        }
        let leap = tai_minus_utc(self.year, self.month, self.day, self.fraction_of_day())?;
        if self.second == 60 && leap.leap_at_end != 1 {
            return Err(Error::InvalidTime);
        }
        let unix_days = unix_days_from_civil(self.year, self.month, self.day);
        let days_from_tai_epoch = (unix_days - TAI_EPOCH_UNIX_DAYS) as i128;
        let mut sod_ns = (self.hour as i128) * 3600 * NS_PER_SEC
            + (self.minute as i128) * 60 * NS_PER_SEC
            + (self.second as i128) * NS_PER_SEC
            + self.nanosecond as i128;
        if self.second == 60 {
            // 23:59:60 is the 86400th second of a leap day (0-based sod = 86400).
            sod_ns = 86_400 * NS_PER_SEC + self.nanosecond as i128;
        }
        let dat_ns = Duration::from_seconds_f64(leap.tai_minus_utc)?.as_nanos();
        let tai_ns = days_from_tai_epoch
            .checked_mul(NS_PER_DAY)
            .and_then(|d| d.checked_add(sod_ns))
            .and_then(|d| d.checked_add(dat_ns))
            .ok_or(Error::Overflow)?;
        Ok(Instant::from_tai_nanos(tai_ns))
    }

    fn fraction_of_day(self) -> f64 {
        let s = self.hour as f64 * 3600.0
            + self.minute as f64 * 60.0
            + self.second.min(59) as f64
            + self.nanosecond as f64 / 1e9;
        s / 86_400.0
    }
}

impl Instant {
    /// UTC civil time. `second` may be 60 on a leap-second day.
    pub fn to_utc(self) -> Result<CivilUtc> {
        let tai_ns = self.as_tai_nanos();
        let approx = tai_ns.saturating_sub(37 * NS_PER_SEC);
        let mut day_index = approx / NS_PER_DAY;
        let sod0 = approx % NS_PER_DAY;
        if sod0 < 0 {
            day_index -= 1;
        }
        let unix0 = day_index as i64 + TAI_EPOCH_UNIX_DAYS;
        for delta in [-2, -1, 0, 1, 2] {
            let unix_days = unix0 + delta;
            let (y, m, d) = civil_from_unix_days(unix_days);
            if y < 1960 {
                continue;
            }
            let leap = match tai_minus_utc(y, m, d, 0.0) {
                Ok(l) => l,
                Err(Error::LeapTableExpired) => continue,
                Err(e) => return Err(e),
            };
            let dat_ns = Duration::from_seconds_f64(leap.tai_minus_utc)?.as_nanos();
            let idx = (unix_days - TAI_EPOCH_UNIX_DAYS) as i128;
            let midnight_tai = idx
                .checked_mul(NS_PER_DAY)
                .and_then(|v| v.checked_add(dat_ns))
                .ok_or(Error::Overflow)?;
            let since = tai_ns.checked_sub(midnight_tai).ok_or(Error::Overflow)?;
            let day_len = NS_PER_DAY + i128::from(leap.leap_at_end) * NS_PER_SEC;
            if since >= 0 && since < day_len {
                return civil_from_since(y, m, d, since, leap.leap_at_end);
            }
        }
        Err(Error::UtcUndefined)
    }
}

fn civil_from_since(y: i32, m: u8, d: u8, since: i128, leap_at_end: i8) -> Result<CivilUtc> {
    if leap_at_end == 1 && since >= NS_PER_DAY {
        let nano = (since - NS_PER_DAY) as u32;
        return Ok(CivilUtc {
            year: y,
            month: m,
            day: d,
            hour: 23,
            minute: 59,
            second: 60,
            nanosecond: nano,
        });
    }
    let hour = (since / (3600 * NS_PER_SEC)) as u8;
    let rem = since % (3600 * NS_PER_SEC);
    let minute = (rem / (60 * NS_PER_SEC)) as u8;
    let rem = rem % (60 * NS_PER_SEC);
    let second = (rem / NS_PER_SEC) as u8;
    let nanosecond = (rem % NS_PER_SEC) as u32;
    CivilUtc::new(y, m, d, hour, minute, second, nanosecond)
}

pub(crate) fn days_in_month(year: i32, month: u8) -> Result<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
        4 | 6 | 9 | 11 => Ok(30),
        2 => Ok(if is_leap_year(year) { 29 } else { 28 }),
        _ => Err(Error::InvalidDate),
    }
}

pub(crate) const fn is_leap_year(year: i32) -> bool {
    let y = year;
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erfa_utc_to_tt_example() {
        // SOFA cookbook: 2010-07-24 11:18:07.318 UTC → 11:19:13.502 TT
        let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
        let inst = c.to_instant().unwrap();
        let tt_ns = inst.reading_tt().as_nanos();
        let tai_days = (unix_days_from_civil(2010, 7, 24) - TAI_EPOCH_UNIX_DAYS) as i128;
        let expected_tt =
            tai_days * NS_PER_DAY + ((11 * 3600 + 19 * 60 + 13) as i128) * NS_PER_SEC + 502_000_000;
        assert_eq!(tt_ns, expected_tt);
    }

    #[test]
    fn leap_second_2016() {
        let leap = CivilUtc::new(2016, 12, 31, 23, 59, 60, 0).unwrap();
        let inst = leap.to_instant().unwrap();
        let next = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert_eq!(next.duration_since(inst).unwrap(), Duration::SECOND);
        let back = inst.to_utc().unwrap();
        assert_eq!(back.second, 60);
        assert_eq!(back.year, 2016);
        assert_eq!(back.month, 12);
        assert_eq!(back.day, 31);
    }

    #[test]
    fn after_leap() {
        let c = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0).unwrap();
        let inst = c.to_instant().unwrap();
        let back = inst.to_utc().unwrap();
        assert_eq!(back, c);
        let leap = tai_minus_utc(2017, 1, 1, 0.0).unwrap();
        assert_eq!(leap.tai_minus_utc, 37.0);
    }
}
