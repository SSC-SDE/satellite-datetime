//! Cached UTC calendar day for hot `Instant` → [`CivilUtc`] without the search loop.
//!
//! Build a [`UtcDay`] once per UTC date, then call [`UtcDay::civil_from_instant`] on every
//! sample that falls on that day. Use [`UtcContext`] when the day is not known in advance.

use crate::constants::{NS_PER_DAY, NS_PER_SEC, TAI_EPOCH_UNIX_DAYS};
use crate::duration::Duration;
use crate::error::{Error, Result};
use crate::instant::Instant;
use crate::julian::unix_days_from_civil;

use super::{civil_from_since, tai_minus_utc, CivilUtc};

/// Leap-second-aware bounds for one proleptic Gregorian UTC date.
///
/// Stores midnight TAI and day length so [`Self::civil_from_instant`] is O(1) arithmetic
/// (no ±2-day search). Build once per day in a hot loop; refresh when [`UtcContext`] misses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UtcDay {
    year: i32,
    month: u8,
    day: u8,
    midnight_tai: i128,
    day_len_ns: i128,
    leap_at_end: i8,
}

impl UtcDay {
    /// Pin one UTC civil date (0h–24h, including a positive leap second at the end if any).
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self> {
        if year < 1960 {
            return Err(Error::UtcUndefined);
        }
        let leap = tai_minus_utc(year, month, day, 0.0)?;
        let unix_days = unix_days_from_civil(year, month, day);
        let idx = (unix_days - TAI_EPOCH_UNIX_DAYS) as i128;
        let dat_ns = Duration::from_seconds_f64(leap.tai_minus_utc)?.as_nanos();
        let midnight_tai = idx
            .checked_mul(NS_PER_DAY)
            .and_then(|v| v.checked_add(dat_ns))
            .ok_or(Error::Overflow)?;
        let day_len_ns = NS_PER_DAY + i128::from(leap.leap_at_end) * NS_PER_SEC;
        Ok(Self {
            year,
            month,
            day,
            midnight_tai,
            day_len_ns,
            leap_at_end: leap.leap_at_end,
        })
    }

    /// Civil year of this pinned day.
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Civil month `1..=12`.
    pub const fn month(self) -> u8 {
        self.month
    }

    /// Civil day `1..=31`.
    pub const fn day(self) -> u8 {
        self.day
    }

    /// TAI nanoseconds at 0h UTC on this day.
    pub const fn midnight_tai(self) -> i128 {
        self.midnight_tai
    }

    /// Whether `instant` falls in this UTC day (including `23:59:60` when applicable).
    pub fn contains(self, instant: Instant) -> bool {
        let tai_ns = instant.as_tai_nanos();
        let since = tai_ns - self.midnight_tai;
        since >= 0 && since < self.day_len_ns
    }

    /// Map `instant` to civil UTC when it lies on this pinned day.
    ///
    /// Returns [`Error::InvalidTime`] if `instant` is outside this day.
    pub fn civil_from_instant(self, instant: Instant) -> Result<CivilUtc> {
        let tai_ns = instant.as_tai_nanos();
        let since = tai_ns
            .checked_sub(self.midnight_tai)
            .ok_or(Error::InvalidTime)?;
        if since < 0 || since >= self.day_len_ns {
            return Err(Error::InvalidTime);
        }
        civil_from_since(self.year, self.month, self.day, since, self.leap_at_end)
    }
}

/// Streaming decoder: fast same-day path, slow [`Instant::to_utc`] only on day change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UtcContext {
    day: UtcDay,
}

impl UtcContext {
    /// Start with a pinned UTC day (typically from the first sample or schedule).
    pub fn new(day: UtcDay) -> Self {
        Self { day }
    }

    /// Current pinned day.
    pub const fn day(self) -> UtcDay {
        self.day
    }

    /// Resolve civil UTC, re-pinning the day when the instant crosses midnight (or a leap).
    pub fn civil_from_instant(&mut self, instant: Instant) -> Result<CivilUtc> {
        if self.day.contains(instant) {
            return self.day.civil_from_instant(instant);
        }
        let utc = instant.to_utc()?;
        self.day = UtcDay::new(utc.year, utc.month, utc.day)?;
        Ok(utc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_day_matches_to_utc() {
        let day = UtcDay::new(2010, 7, 24).unwrap();
        let civil = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
        let inst = civil.to_instant().unwrap();
        assert!(day.contains(inst));
        assert_eq!(day.civil_from_instant(inst).unwrap(), civil);
        assert_eq!(inst.to_utc().unwrap(), civil);
    }

    #[test]
    fn leap_second_day() {
        let day = UtcDay::new(2016, 12, 31).unwrap();
        let leap = CivilUtc::new(2016, 12, 31, 23, 59, 60, 0).unwrap();
        let inst = leap.to_instant().unwrap();
        assert!(day.contains(inst));
        assert_eq!(day.civil_from_instant(inst).unwrap(), leap);
    }

    #[test]
    fn outside_day_errors() {
        let day = UtcDay::new(2017, 1, 1).unwrap();
        let prev = CivilUtc::new(2016, 12, 31, 23, 59, 59, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert!(!day.contains(prev));
        assert_eq!(day.civil_from_instant(prev), Err(Error::InvalidTime));
    }

    #[test]
    fn context_repins_on_crossing() {
        let day0 = UtcDay::new(2016, 12, 31).unwrap();
        let mut ctx = UtcContext::new(day0);
        let leap = CivilUtc::new(2016, 12, 31, 23, 59, 60, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert_eq!(ctx.civil_from_instant(leap).unwrap().second, 60);
        let next = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        let c = ctx.civil_from_instant(next).unwrap();
        assert_eq!(c.year, 2017);
        assert_eq!(ctx.day().year(), 2017);
    }

    #[test]
    fn many_samples_same_day() {
        let day = UtcDay::new(2010, 7, 24).unwrap();
        let base = day.midnight_tai();
        for sod in [0, 3600, 86_399] {
            let inst = Instant::from_tai_nanos(base + sod as i128 * NS_PER_SEC);
            let fast = day.civil_from_instant(inst).unwrap();
            let slow = inst.to_utc().unwrap();
            assert_eq!(fast, slow);
        }
    }
}
