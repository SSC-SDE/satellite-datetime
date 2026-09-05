//! Two-part Julian dates and day counts.

use crate::constants::{NS_PER_DAY, TAI_EPOCH_JD, TAI_EPOCH_MJD};
use crate::instant::Instant;

/// Split a Julian Date into (day, fraction of day from midnight).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JulianDate {
    /// Integer + 0.5 style first part (SOFA: typically the whole JD at 0h or a large part).
    pub d1: f64,
    /// Remainder so that `d1 + d2` is the Julian Date.
    pub d2: f64,
}

impl JulianDate {
    /// Sum as `f64` (≈50 µs ULP near J2000).
    pub fn as_f64(self) -> f64 {
        self.d1 + self.d2
    }

    /// Modified Julian Date (`JD − 2400000.5`).
    pub fn mjd(self) -> f64 {
        self.as_f64() - 2_400_000.5
    }
}

impl Instant {
    /// Julian Date on TAI, two-part (large integer day + fraction) for ERFA-style tests.
    pub fn julian_tai(self) -> JulianDate {
        let ns = self.as_tai_nanos();
        let days = (ns / NS_PER_DAY) as f64;
        let rem = (ns % NS_PER_DAY) as f64 / NS_PER_DAY as f64;
        JulianDate {
            d1: TAI_EPOCH_JD + days,
            d2: rem,
        }
    }

    /// Julian Date on TT.
    pub fn julian_tt(self) -> JulianDate {
        let ns = self.reading_tt().as_nanos();
        let days = (ns / NS_PER_DAY) as f64;
        let rem = (ns % NS_PER_DAY) as f64 / NS_PER_DAY as f64;
        JulianDate {
            d1: TAI_EPOCH_JD + days,
            d2: rem,
        }
    }

    /// MJD(TAI) as integer day + nanoseconds of day.
    pub fn mjd_tai_parts(self) -> (i64, i128) {
        let ns = self.as_tai_nanos();
        let day = TAI_EPOCH_MJD + (ns / NS_PER_DAY) as i64;
        let rem = ns % NS_PER_DAY;
        (day, rem)
    }
}

/// Proleptic Gregorian civil date to Unix day number (days since 1970-01-01).
/// Howard Hinnant's public-domain algorithm.
pub const fn unix_days_from_civil(y: i32, m: u8, d: u8) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if m > 2 {
        (m as u32) - 3
    } else {
        (m as u32) + 9
    };
    let doy = (153 * mp + 2) / 5 + d as u32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

/// Inverse of [`unix_days_from_civil`].
pub const fn civil_from_unix_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i32) + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u8, d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_epoch_and_tai_epoch() {
        assert_eq!(unix_days_from_civil(1970, 1, 1), 0);
        assert_eq!(unix_days_from_civil(1958, 1, 1), crate::TAI_EPOCH_UNIX_DAYS);
        assert_eq!(civil_from_unix_days(0), (1970, 1, 1));
        assert_eq!(
            civil_from_unix_days(crate::TAI_EPOCH_UNIX_DAYS),
            (1958, 1, 1)
        );
    }

    #[test]
    fn j2000_tt_jd() {
        // 2000-01-01 12:00:00 TT = JD 2451545.0
        // TT = TAI + 32.184 s → 12:00:00 TT = 11:59:27.816 TAI
        let civil_ok = unix_days_from_civil(2000, 1, 1);
        let tai_days = civil_ok - crate::TAI_EPOCH_UNIX_DAYS;
        let noon_minus = 12 * 3600 * crate::NS_PER_SEC - 32_184_000_000;
        let inst = Instant::from_tai_nanos(tai_days as i128 * NS_PER_DAY + noon_minus);
        let jd = inst.julian_tt().as_f64();
        assert!((jd - 2_451_545.0).abs() < 1e-8, "jd={jd}");
    }
}
