//! CCSDS 301.0-B-4 time codes. Recommended epoch: 1958-01-01 TAI (no leap seconds).

use crate::constants::NS_PER_SEC;
use crate::error::{Error, Result};
use crate::instant::Instant;

/// CUC configuration: 1–4 coarse octets (seconds) and 0–3 fine octets (fraction).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CucConfig {
    /// Number of octets counting whole SI seconds from the TAI 1958 epoch (`1..=4`).
    pub coarse_octets: u8,
    /// Number of octets of binary fraction of a second (`0..=3`).
    pub fine_octets: u8,
}

impl CucConfig {
    /// 4-byte seconds + 2-byte fraction (~15.3 µs), a common agency choice.
    pub const C4_F2: Self = Self {
        coarse_octets: 4,
        fine_octets: 2,
    };

    fn t_len(self) -> Result<usize> {
        if !(1..=4).contains(&self.coarse_octets) || self.fine_octets > 3 {
            return Err(Error::Codec);
        }
        Ok((self.coarse_octets + self.fine_octets) as usize)
    }
}

/// Encode TAI since 1958-01-01 as CCSDS unsegmented time code (T-field only).
pub fn encode_cuc(instant: Instant, cfg: CucConfig, out: &mut [u8]) -> Result<usize> {
    let n = cfg.t_len()?;
    if out.len() < n {
        return Err(Error::BufferTooSmall);
    }
    let tai_ns = instant.as_tai_nanos();
    if tai_ns < 0 {
        return Err(Error::Codec);
    }
    let coarse = (tai_ns / NS_PER_SEC) as u64;
    let frac_ns = (tai_ns % NS_PER_SEC) as u64;
    let coarse_max = (1u64 << (8 * cfg.coarse_octets as u32)) - 1;
    if coarse > coarse_max {
        return Err(Error::Overflow);
    }
    let mut idx = 0;
    for k in (0..cfg.coarse_octets).rev() {
        out[idx] = ((coarse >> (8 * k)) & 0xff) as u8;
        idx += 1;
    }
    // Fine field: binary fraction of a second, each octet is 1/256 of the previous.
    let mut rem = frac_ns;
    for _ in 0..cfg.fine_octets {
        rem *= 256;
        let byte = (rem / NS_PER_SEC as u64) as u8;
        rem %= NS_PER_SEC as u64;
        out[idx] = byte;
        idx += 1;
    }
    Ok(n)
}

/// Decode a CUC T-field.
pub fn decode_cuc(buf: &[u8], cfg: CucConfig) -> Result<Instant> {
    let n = cfg.t_len()?;
    if buf.len() < n {
        return Err(Error::Codec);
    }
    let mut coarse: u64 = 0;
    let mut i = 0;
    for _ in 0..cfg.coarse_octets {
        coarse = (coarse << 8) | buf[i] as u64;
        i += 1;
    }
    let mut frac_ns: u64 = 0;
    let mut num = 0u64;
    let mut den = 1u64;
    for _ in 0..cfg.fine_octets {
        num = (num << 8) | buf[i] as u64;
        den <<= 8;
        i += 1;
    }
    if cfg.fine_octets > 0 {
        frac_ns = ((num as u128 * NS_PER_SEC as u128) / den as u128) as u64;
    }
    let tai_ns = (coarse as i128)
        .checked_mul(NS_PER_SEC)
        .and_then(|s| s.checked_add(frac_ns as i128))
        .ok_or(Error::Overflow)?;
    Ok(Instant::from_tai_nanos(tai_ns))
}

/// CDS: 16-bit day count from 1958-01-01 + 32-bit milliseconds of day.
pub fn encode_cds(instant: Instant, out: &mut [u8]) -> Result<usize> {
    if out.len() < 6 {
        return Err(Error::BufferTooSmall);
    }
    let ns = instant.as_tai_nanos();
    if ns < 0 {
        return Err(Error::Codec);
    }
    let days = ns / crate::NS_PER_DAY;
    let ms = ((ns % crate::NS_PER_DAY) / 1_000_000) as u32;
    if days > u16::MAX as i128 {
        return Err(Error::Overflow);
    }
    let d = days as u16;
    out[0] = (d >> 8) as u8;
    out[1] = d as u8;
    out[2] = (ms >> 24) as u8;
    out[3] = (ms >> 16) as u8;
    out[4] = (ms >> 8) as u8;
    out[5] = ms as u8;
    Ok(6)
}

/// Decode a 6-octet CDS T-field (day + millisecond of day).
pub fn decode_cds(buf: &[u8]) -> Result<Instant> {
    if buf.len() < 6 {
        return Err(Error::Codec);
    }
    let days = u16::from_be_bytes([buf[0], buf[1]]) as i128;
    let ms = u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]) as i128;
    if ms >= 86_400_000 {
        return Err(Error::InvalidTime);
    }
    Ok(Instant::from_tai_nanos(
        days * crate::NS_PER_DAY + ms * 1_000_000,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::duration::Duration;

    #[test]
    fn cuc_roundtrip() {
        let t = Instant::TAI_EPOCH
            .checked_add(Duration::from_seconds(1_234_567))
            .unwrap()
            .checked_add(Duration::from_millis(15))
            .unwrap();
        let mut buf = [0u8; 8];
        let n = encode_cuc(t, CucConfig::C4_F2, &mut buf).unwrap();
        let back = decode_cuc(&buf[..n], CucConfig::C4_F2).unwrap();
        let err = t.duration_since(back).unwrap().as_nanos().abs();
        assert!(err < 20_000, "err ns {err}"); // 2-byte fraction ~15µs
    }

    #[test]
    fn cds_roundtrip() {
        let t = Instant::from_tai_nanos(10_000 * crate::NS_PER_DAY + 43_200_000 * 1_000_000);
        let mut buf = [0u8; 6];
        encode_cds(t, &mut buf).unwrap();
        assert_eq!(decode_cds(&buf).unwrap(), t);
    }
}
