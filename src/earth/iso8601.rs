//! ISO 8601 / RFC 3339 parse and format. Leap second `23:59:60` is accepted.

use crate::earth::CivilUtc;
use crate::error::{Error, Result};
use crate::instant::Instant;

/// Parse `YYYY-MM-DDTHH:MM:SS[.frac]Z` or with `±HH:MM` offset (interpreted as UTC after applying offset).
pub fn parse_rfc3339(s: &str) -> Result<Instant> {
    let b = s.as_bytes();
    if b.len() < 20 {
        return Err(Error::Parse);
    }
    let year = parse_n(&b[0..4])? as i32;
    if b[4] != b'-' || b[7] != b'-' {
        return Err(Error::Parse);
    }
    let month = parse_n(&b[5..7])? as u8;
    let day = parse_n(&b[8..10])? as u8;
    if b[10] != b'T' && b[10] != b't' && b[10] != b' ' {
        return Err(Error::Parse);
    }
    let hour = parse_n(&b[11..13])? as u8;
    if b[13] != b':' {
        return Err(Error::Parse);
    }
    let minute = parse_n(&b[14..16])? as u8;
    if b[16] != b':' {
        return Err(Error::Parse);
    }
    let second = parse_n(&b[17..19])? as u8;
    let mut i = 19;
    let mut nanosecond = 0u32;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        let frac = core::str::from_utf8(&b[start..i]).map_err(|_| Error::Parse)?;
        if frac.is_empty() || frac.len() > 9 {
            return Err(Error::Parse);
        }
        let mut f = 0u32;
        for (k, ch) in frac.bytes().enumerate() {
            f += (ch - b'0') as u32 * 10u32.pow(8 - k as u32);
        }
        nanosecond = f;
    }
    if i >= b.len() {
        return Err(Error::Parse);
    }
    let (off_h, off_m, sign) = if b[i] == b'Z' || b[i] == b'z' {
        if i + 1 != b.len() {
            return Err(Error::Parse);
        }
        (0, 0, 1i32)
    } else if b[i] == b'+' || b[i] == b'-' {
        let sign = if b[i] == b'+' { 1i32 } else { -1 };
        i += 1;
        if i + 2 > b.len() {
            return Err(Error::Parse);
        }
        let oh = parse_n(&b[i..i + 2])? as i32;
        i += 2;
        let om = if i < b.len() && b[i] == b':' {
            i += 1;
            if i + 2 > b.len() {
                return Err(Error::Parse);
            }
            let m = parse_n(&b[i..i + 2])? as i32;
            i += 2;
            m
        } else if i + 2 <= b.len() && b[i].is_ascii_digit() {
            let m = parse_n(&b[i..i + 2])? as i32;
            i += 2;
            m
        } else {
            0
        };
        if i != b.len() {
            return Err(Error::Parse);
        }
        (oh, om, sign)
    } else {
        return Err(Error::Parse);
    };
    let civil = CivilUtc::new(year, month, day, hour, minute, second, nanosecond)?;
    let inst = civil.to_instant()?;
    let off = sign * (off_h * 3600 + off_m * 60);
    inst.checked_sub(crate::Duration::from_seconds(off as i64))
}

fn parse_n(b: &[u8]) -> Result<u32> {
    let mut n = 0u32;
    if b.is_empty() {
        return Err(Error::Parse);
    }
    for c in b {
        if !c.is_ascii_digit() {
            return Err(Error::Parse);
        }
        n = n * 10 + (*c - b'0') as u32;
    }
    Ok(n)
}

/// Write RFC 3339 UTC (`...Z`) into `buf`. Returns bytes written.
pub fn format_rfc3339(instant: Instant, buf: &mut [u8]) -> Result<usize> {
    let c = instant.to_utc()?;
    format_civil_z(c, buf)
}

pub(crate) fn format_civil_z(c: CivilUtc, buf: &mut [u8]) -> Result<usize> {
    //  YYYY-MM-DDTHH:MM:SS.nnnnnnnnnZ  = 30 chars with 9-digit frac
    const NEED: usize = 30;
    if buf.len() < NEED {
        return Err(Error::BufferTooSmall);
    }
    write_i32(buf, 0, 4, c.year);
    buf[4] = b'-';
    write_u32(buf, 5, 2, c.month as u32);
    buf[7] = b'-';
    write_u32(buf, 8, 2, c.day as u32);
    buf[10] = b'T';
    write_u32(buf, 11, 2, c.hour as u32);
    buf[13] = b':';
    write_u32(buf, 14, 2, c.minute as u32);
    buf[16] = b':';
    write_u32(buf, 17, 2, c.second as u32);
    buf[19] = b'.';
    write_u32(buf, 20, 9, c.nanosecond);
    buf[29] = b'Z';
    Ok(NEED)
}

fn write_u32(buf: &mut [u8], start: usize, width: usize, mut v: u32) {
    for i in (0..width).rev() {
        buf[start + i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
}

fn write_i32(buf: &mut [u8], start: usize, width: usize, v: i32) {
    write_u32(buf, start, width, v.unsigned_abs());
    if v < 0 && width > 0 {
        buf[start] = b'-';
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::earth::CivilUtc;

    #[test]
    fn roundtrip_z() {
        let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
        let inst = c.to_instant().unwrap();
        let mut buf = [0u8; 32];
        let n = format_rfc3339(inst, &mut buf).unwrap();
        let s = core::str::from_utf8(&buf[..n]).unwrap();
        let back = parse_rfc3339(s).unwrap();
        assert_eq!(back, inst);
    }

    #[test]
    fn parse_leap() {
        let inst = parse_rfc3339("2016-12-31T23:59:60Z").unwrap();
        assert_eq!(inst.to_utc().unwrap().second, 60);
    }

    #[test]
    fn parse_offset() {
        let a = parse_rfc3339("2010-07-24T11:18:07.318Z").unwrap();
        let b = parse_rfc3339("2010-07-24T07:18:07.318-04:00").unwrap();
        assert_eq!(a, b);
    }
}
