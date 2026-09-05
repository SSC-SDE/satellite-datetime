//! TAI−UTC from IERS Bulletin C / USNO tai-utc.dat (numeric table, not SOFA source).
//!
//! Last leap second in this pin: 2016-12-31. Dates more than five years past 2023
//! return [`Error::LeapTableExpired`] as a warning analog to SOFA `eraDat` (we
//! error rather than silently extrapolate).

use crate::error::{Error, Result};

/// Result of a TAI−UTC lookup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeapInfo {
    /// TAI − UTC in SI seconds (may be non-integer before 1972).
    pub tai_minus_utc: f64,
    /// +1 if this UTC day *ends* with a positive leap second, 0 otherwise.
    pub leap_at_end: i8,
    /// Bulletin / table version pin.
    pub table: &'static str,
}

/// Version string for the numeric leap-second table in this crate build.
pub const LEAP_TABLE_VERSION: &str = "IERS-Bulletin-C-2016-12-31+SOFA-pre1972-numeric";
const RELEASE_YEAR: i32 = 2023;

struct Change {
    year: i32,
    month: u8,
    delta: f64,
}

/// Pre-1972 MJD reference and drift (s/day), aligned with IERS tai-utc.dat.
struct Drift {
    mjd0: f64,
    rate: f64,
}

const CHANGES: &[Change] = &[
    Change {
        year: 1960,
        month: 1,
        delta: 1.4178180,
    },
    Change {
        year: 1961,
        month: 1,
        delta: 1.4228180,
    },
    Change {
        year: 1961,
        month: 8,
        delta: 1.3728180,
    },
    Change {
        year: 1962,
        month: 1,
        delta: 1.8458580,
    },
    Change {
        year: 1963,
        month: 11,
        delta: 1.9458580,
    },
    Change {
        year: 1964,
        month: 1,
        delta: 3.2401300,
    },
    Change {
        year: 1964,
        month: 4,
        delta: 3.3401300,
    },
    Change {
        year: 1964,
        month: 9,
        delta: 3.4401300,
    },
    Change {
        year: 1965,
        month: 1,
        delta: 3.5401300,
    },
    Change {
        year: 1965,
        month: 3,
        delta: 3.6401300,
    },
    Change {
        year: 1965,
        month: 7,
        delta: 3.7401300,
    },
    Change {
        year: 1965,
        month: 9,
        delta: 3.8401300,
    },
    Change {
        year: 1966,
        month: 1,
        delta: 4.3131700,
    },
    Change {
        year: 1968,
        month: 2,
        delta: 4.2131700,
    },
    Change {
        year: 1972,
        month: 1,
        delta: 10.0,
    },
    Change {
        year: 1972,
        month: 7,
        delta: 11.0,
    },
    Change {
        year: 1973,
        month: 1,
        delta: 12.0,
    },
    Change {
        year: 1974,
        month: 1,
        delta: 13.0,
    },
    Change {
        year: 1975,
        month: 1,
        delta: 14.0,
    },
    Change {
        year: 1976,
        month: 1,
        delta: 15.0,
    },
    Change {
        year: 1977,
        month: 1,
        delta: 16.0,
    },
    Change {
        year: 1978,
        month: 1,
        delta: 17.0,
    },
    Change {
        year: 1979,
        month: 1,
        delta: 18.0,
    },
    Change {
        year: 1980,
        month: 1,
        delta: 19.0,
    },
    Change {
        year: 1981,
        month: 7,
        delta: 20.0,
    },
    Change {
        year: 1982,
        month: 7,
        delta: 21.0,
    },
    Change {
        year: 1983,
        month: 7,
        delta: 22.0,
    },
    Change {
        year: 1985,
        month: 7,
        delta: 23.0,
    },
    Change {
        year: 1988,
        month: 1,
        delta: 24.0,
    },
    Change {
        year: 1990,
        month: 1,
        delta: 25.0,
    },
    Change {
        year: 1991,
        month: 1,
        delta: 26.0,
    },
    Change {
        year: 1992,
        month: 7,
        delta: 27.0,
    },
    Change {
        year: 1993,
        month: 7,
        delta: 28.0,
    },
    Change {
        year: 1994,
        month: 7,
        delta: 29.0,
    },
    Change {
        year: 1996,
        month: 1,
        delta: 30.0,
    },
    Change {
        year: 1997,
        month: 7,
        delta: 31.0,
    },
    Change {
        year: 1999,
        month: 1,
        delta: 32.0,
    },
    Change {
        year: 2006,
        month: 1,
        delta: 33.0,
    },
    Change {
        year: 2009,
        month: 1,
        delta: 34.0,
    },
    Change {
        year: 2012,
        month: 7,
        delta: 35.0,
    },
    Change {
        year: 2015,
        month: 7,
        delta: 36.0,
    },
    Change {
        year: 2017,
        month: 1,
        delta: 37.0,
    },
];

const NERA1: usize = 14;

const DRIFT: &[Drift] = &[
    Drift {
        mjd0: 37300.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 37300.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 37300.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 37665.0,
        rate: 0.0011232,
    },
    Drift {
        mjd0: 37665.0,
        rate: 0.0011232,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 38761.0,
        rate: 0.0012960,
    },
    Drift {
        mjd0: 39126.0,
        rate: 0.0025920,
    },
    Drift {
        mjd0: 39126.0,
        rate: 0.0025920,
    },
];

/// TAI − UTC at 0h of the given UTC date (`fd` is fraction of day, used pre-1972).
pub fn tai_minus_utc(year: i32, month: u8, day: u8, fd: f64) -> Result<LeapInfo> {
    if year < 1960 {
        return Err(Error::UtcUndefined);
    }
    if year > RELEASE_YEAR + 5 {
        return Err(Error::LeapTableExpired);
    }
    if !(0.0..=1.0).contains(&fd) {
        return Err(Error::InvalidTime);
    }
    let m = 12 * year + month as i32;
    let mut idx = 0;
    for (i, ch) in CHANGES.iter().enumerate().rev() {
        if m >= 12 * ch.year + ch.month as i32 {
            idx = i;
            break;
        }
    }
    let mut da = CHANGES[idx].delta;
    if idx < NERA1 {
        let mjd = crate::julian::unix_days_from_civil(year, month, day) as f64 + 40587.0;
        da += (mjd + fd - DRIFT[idx].mjd0) * DRIFT[idx].rate;
    }
    let leap_at_end = leap_seconds_on_utc_day(year, month, day)?;
    Ok(LeapInfo {
        tai_minus_utc: da,
        leap_at_end,
        table: LEAP_TABLE_VERSION,
    })
}

/// +1 if a positive leap second is inserted at the end of this UTC day.
pub fn leap_seconds_on_utc_day(year: i32, month: u8, day: u8) -> Result<i8> {
    let dim = crate::earth::days_in_month(year, month)?;
    if day != dim {
        return Ok(0);
    }
    // Leap seconds occur at the end of June or December in the modern era.
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else if month == 6 {
        (year, 7)
    } else {
        return Ok(0);
    };
    let now = lookup_delta(year, month);
    let nxt = lookup_delta(ny, nm);
    Ok((nxt - now) as i8)
}

fn lookup_delta(year: i32, month: u8) -> f64 {
    let m = 12 * year + month as i32;
    let mut da = CHANGES[0].delta;
    for ch in CHANGES {
        if m >= 12 * ch.year + ch.month as i32 {
            da = ch.delta;
        }
    }
    da
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_leap_offset() {
        assert_eq!(
            tai_minus_utc(2016, 12, 31, 0.0).unwrap().tai_minus_utc,
            36.0
        );
        assert_eq!(tai_minus_utc(2017, 1, 1, 0.0).unwrap().tai_minus_utc, 37.0);
        assert_eq!(leap_seconds_on_utc_day(2016, 12, 31).unwrap(), 1);
        assert_eq!(leap_seconds_on_utc_day(2016, 12, 30).unwrap(), 0);
    }

    #[test]
    fn gps_epoch_offset() {
        // 1980-01-06: TAI−UTC = 19
        assert_eq!(tai_minus_utc(1980, 1, 6, 0.0).unwrap().tai_minus_utc, 19.0);
    }
}
