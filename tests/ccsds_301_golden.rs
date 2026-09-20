//! Numeric CCSDS 301.0-B-4 wire examples (P-field + T-field bytes only).

use satellite_datetime::ccsds::{
    decode_cds_with_p, decode_cuc_with_p, encode_cds_with_p, encode_cuc_with_p, CucConfig,
};
use satellite_datetime::Instant;

/// Level-1 CUC, 4 coarse + 2 fine octets, TAI 1958 epoch: P-field `0x1E` (§3.2.2).
#[test]
fn cuc_p_tai_epoch_zero() {
    let wire = [0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let (t, cfg) = decode_cuc_with_p(&wire).unwrap();
    assert_eq!(cfg, CucConfig::C4_F2);
    assert_eq!(t, Instant::TAI_EPOCH);
    let mut out = [0u8; 8];
    let n = encode_cuc_with_p(t, cfg, &mut out).unwrap();
    assert_eq!(&out[..n], &wire);
}

/// One TAI second after the 1958 epoch with zero fractional field.
#[test]
fn cuc_p_one_second() {
    let wire = [0x1E, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];
    let (t, cfg) = decode_cuc_with_p(&wire).unwrap();
    assert_eq!(t.as_tai_nanos(), 1_000_000_000);
    let mut out = [0u8; 8];
    encode_cuc_with_p(t, cfg, &mut out).unwrap();
    assert_eq!(&out[..7], &wire);
}

/// CDS level-1, 16-bit day + millisecond-of-day: P-field `0x40` (§3.3.2).
#[test]
fn cds_p_day_and_ms() {
    let wire = [0x40, 0x27, 0x10, 0x00, 0x01, 0x86, 0xA0];
    let t = decode_cds_with_p(&wire).unwrap();
    assert_eq!(
        t.as_tai_nanos(),
        10_000 * satellite_datetime::NS_PER_DAY + 100_000 * 1_000_000,
    );
    let mut out = [0u8; 8];
    let n = encode_cds_with_p(t, &mut out).unwrap();
    assert_eq!(&out[..n], &wire);
}
