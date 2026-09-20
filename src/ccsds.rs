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

/// One-octet CCSDS P-field for level-1 CUC (1958-01-01 TAI epoch).
///
/// Bit layout (MSB = bit 7): extension (0), time code ID `001`, coarse length minus
/// one, fractional octet count. Matches CCSDS 301.0-B-4 §3.2.2 without a second P-octet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CucPField {
    /// Coarse/fine layout implied by the preamble.
    pub config: CucConfig,
}

const CUC_ID_TAI_1958: u8 = 0x01;
const CUC_ID_AGENCY: u8 = 0x02;
/// CDS level-1, 1958 epoch, 16-bit day, millisecond-of-day (no sub-millisecond segment).
const CDS_P_DAY16_MS32: u8 = 0x40;

impl CucPField {
    /// Preamble for [`CucConfig::C4_F2`] (`0x1E`).
    pub const C4_F2: Self = Self {
        config: CucConfig::C4_F2,
    };

    /// Build a preamble for a supported coarse/fine layout.
    pub fn new(config: CucConfig) -> Result<Self> {
        config.t_len()?;
        Ok(Self { config })
    }

    /// Parse a one-octet CUC P-field from the wire.
    pub fn from_octet(octet: u8) -> Result<Self> {
        if octet & 0x80 != 0 {
            return Err(Error::Unsupported);
        }
        let id = (octet >> 4) & 0x07;
        match id {
            CUC_ID_TAI_1958 => {}
            CUC_ID_AGENCY => return Err(Error::Unsupported),
            _ => return Err(Error::Codec),
        }
        let coarse_minus_1 = (octet >> 2) & 0x03;
        let fine = octet & 0x03;
        let config = CucConfig {
            coarse_octets: coarse_minus_1 + 1,
            fine_octets: fine,
        };
        config.t_len()?;
        Ok(Self { config })
    }

    /// Encode this preamble to one octet.
    pub fn to_octet(self) -> u8 {
        let c = self.config;
        (CUC_ID_TAI_1958 << 4) | ((c.coarse_octets - 1) << 2) | c.fine_octets
    }
}

fn decode_cds_p_octet(octet: u8) -> Result<()> {
    if octet & 0x80 != 0 {
        return Err(Error::Unsupported);
    }
    let id = (octet >> 4) & 0x07;
    match id {
        0x04 => {}
        0x05 => return Err(Error::Unsupported),
        _ => return Err(Error::Codec),
    }
    if octet & 0x0F != 0 {
        return Err(Error::Unsupported);
    }
    Ok(())
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

/// Encode CUC P-field + T-field (`1 + coarse + fine` octets).
pub fn encode_cuc_with_p(instant: Instant, cfg: CucConfig, out: &mut [u8]) -> Result<usize> {
    let p = CucPField::new(cfg)?;
    let t_len = cfg.t_len()?;
    if out.len() < 1 + t_len {
        return Err(Error::BufferTooSmall);
    }
    out[0] = p.to_octet();
    let n = encode_cuc(instant, cfg, &mut out[1..])?;
    Ok(1 + n)
}

/// Decode CUC P-field + T-field; returns the instant and the wire coarse/fine layout.
pub fn decode_cuc_with_p(buf: &[u8]) -> Result<(Instant, CucConfig)> {
    if buf.is_empty() {
        return Err(Error::Codec);
    }
    let p = CucPField::from_octet(buf[0])?;
    let instant = decode_cuc(&buf[1..], p.config)?;
    Ok((instant, p.config))
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

/// Encode CDS P-field + 6-octet T-field (16-bit day + millisecond of day).
pub fn encode_cds_with_p(instant: Instant, out: &mut [u8]) -> Result<usize> {
    if out.len() < 7 {
        return Err(Error::BufferTooSmall);
    }
    out[0] = CDS_P_DAY16_MS32;
    encode_cds(instant, &mut out[1..])?;
    Ok(7)
}

/// Decode CDS P-field + T-field.
pub fn decode_cds_with_p(buf: &[u8]) -> Result<Instant> {
    if buf.len() < 7 {
        return Err(Error::Codec);
    }
    decode_cds_p_octet(buf[0])?;
    decode_cds(&buf[1..])
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

    #[test]
    fn cuc_p_c4f2_octet() {
        assert_eq!(CucPField::C4_F2.to_octet(), 0x1E);
        assert_eq!(
            CucPField::from_octet(0x1E).unwrap().config,
            CucConfig::C4_F2
        );
    }

    #[test]
    fn cuc_with_p_roundtrip() {
        let t = Instant::TAI_EPOCH
            .checked_add(Duration::from_seconds(42))
            .unwrap();
        let mut buf = [0u8; 9];
        let n = encode_cuc_with_p(t, CucConfig::C4_F2, &mut buf).unwrap();
        assert_eq!(n, 7);
        assert_eq!(buf[0], 0x1E);
        let (back, cfg) = decode_cuc_with_p(&buf[..n]).unwrap();
        assert_eq!(cfg, CucConfig::C4_F2);
        assert_eq!(back, t);
    }

    #[test]
    fn cuc_p_rejects_extension_and_agency() {
        assert_eq!(CucPField::from_octet(0x9E).unwrap_err(), Error::Unsupported);
        assert_eq!(CucPField::from_octet(0x2E).unwrap_err(), Error::Unsupported);
    }

    #[test]
    fn cds_with_p_roundtrip() {
        let t = Instant::from_tai_nanos(10_000 * crate::NS_PER_DAY + 43_200_000 * 1_000_000);
        let mut buf = [0u8; 7];
        let n = encode_cds_with_p(t, &mut buf).unwrap();
        assert_eq!(n, 7);
        assert_eq!(buf[0], 0x40);
        assert_eq!(decode_cds_with_p(&buf).unwrap(), t);
    }

    #[test]
    fn cds_p_rejects_agency_layout() {
        assert_eq!(
            decode_cds_with_p(&[0x50, 0, 0, 0, 0, 0, 0]).unwrap_err(),
            Error::Unsupported
        );
    }
}
