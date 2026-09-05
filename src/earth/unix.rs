//! POSIX Unix time (leap seconds not included in the integer) vs elapsed SI.

use crate::constants::{NS_PER_DAY, NS_PER_SEC, TAI_EPOCH_UNIX_DAYS};
use crate::earth::CivilUtc;
use crate::error::{Error, Result};
use crate::instant::Instant;
use crate::julian::unix_days_from_civil;

/// POSIX seconds since 1970-01-01 00:00:00 UTC, **not** counting leap seconds.
pub fn to_posix_seconds(instant: Instant) -> Result<i64> {
    let c = instant.to_utc()?;
    let days = unix_days_from_civil(c.year, c.month, c.day);
    let mut sod = c.hour as i64 * 3600 + c.minute as i64 * 60 + c.second.min(59) as i64;
    if c.second == 60 {
        // POSIX typically repeats 23:59:59 or jumps; we map the leap second to the
        // next day's 00:00:00 Unix value minus 0 (same as 2017-01-01 00:00:00).
        sod = 86400;
    }
    days.checked_mul(86_400)
        .and_then(|d| d.checked_add(sod))
        .ok_or(Error::Overflow)
}

/// Interpret POSIX seconds as UTC civil time (no leap in the day count) and convert to TAI.
pub fn from_posix_seconds(secs: i64) -> Result<Instant> {
    from_posix_nanos(secs as i128 * NS_PER_SEC)
}

/// POSIX nanoseconds (Java `Instant`, JS `Date` × 1e6).
pub fn from_posix_nanos(ns: i128) -> Result<Instant> {
    let mut days = ns / NS_PER_DAY;
    let mut sod = ns % NS_PER_DAY;
    if sod < 0 {
        days -= 1;
        sod += NS_PER_DAY;
    }
    let unix_days = i64::try_from(days).map_err(|_| Error::Overflow)?;
    let (y, m, d) = crate::julian::civil_from_unix_days(unix_days);
    let hour = (sod / (3600 * NS_PER_SEC)) as u8;
    let rem = sod % (3600 * NS_PER_SEC);
    let minute = (rem / (60 * NS_PER_SEC)) as u8;
    let rem = rem % (60 * NS_PER_SEC);
    let second = (rem / NS_PER_SEC) as u8;
    let nanosecond = (rem % NS_PER_SEC) as u32;
    CivilUtc::new(y, m, d, hour, minute, second, nanosecond)?.to_instant()
}

/// True SI nanoseconds from the Unix epoch instant (1970-01-01 00:00:00 UTC) to `instant`.
///
/// This *does* count leap seconds. It disagrees with POSIX by the number of leaps
/// between the epoch and the instant.
pub fn si_nanos_since_unix_epoch(instant: Instant) -> Result<i128> {
    let epoch = CivilUtc::new(1970, 1, 1, 0, 0, 0, 0)?.to_instant()?;
    Ok(instant.duration_since(epoch)?.as_nanos())
}

/// TAI nanoseconds from 1958-01-01 to Unix epoch (computed via the leap table).
#[allow(dead_code)]
pub fn unix_epoch_tai_ns() -> Result<i128> {
    Ok(CivilUtc::new(1970, 1, 1, 0, 0, 0, 0)?
        .to_instant()?
        .as_tai_nanos())
}

const _: i64 = TAI_EPOCH_UNIX_DAYS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earth::CivilUtc;

    #[test]
    fn posix_ignores_leaps() {
        let a = CivilUtc::new(2016, 12, 31, 23, 59, 59, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        let b = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert_eq!(
            to_posix_seconds(b).unwrap() - to_posix_seconds(a).unwrap(),
            1
        );
        assert_eq!(b.duration_since(a).unwrap().as_seconds(), 2);
        let si = si_nanos_since_unix_epoch(b).unwrap() - si_nanos_since_unix_epoch(a).unwrap();
        assert_eq!(si, 2_000_000_000);
    }

    #[test]
    fn posix_2017() {
        let inst = from_posix_seconds(1_483_228_800).unwrap();
        let c = inst.to_utc().unwrap();
        assert_eq!(c.year, 2017);
        assert_eq!(c.month, 1);
        assert_eq!(c.day, 1);
    }
}
