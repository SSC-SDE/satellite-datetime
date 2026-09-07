//! Golden vectors from the IAU SOFA *Time Scale and Calendar Tools* cookbook
//! and IERS TAI−UTC (same numeric table ERFA `eraDat` uses).
//!
//! We do **not** copy SOFA/ERFA source. These are published input/output pairs
//! so UTC↔TAI↔TT can be checked independently of our implementation.

#![cfg(feature = "earth")]

use satellite_datetime::earth::tai_minus_utc;
use satellite_datetime::{
    unix_days_from_civil, CivilUtc, Duration, Instant, NS_PER_DAY, NS_PER_SEC,
};

/// SOFA cookbook §1.5: 2010-07-24 11:18:07.318 UTC → 11:19:13.502 TT
/// (UTC→TAI→TT; TAI−UTC = 34 s that day).
#[test]
fn sofa_cookbook_utc_to_tt_2010_jul_24() {
    let utc = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
    let inst = utc.to_instant().unwrap();
    assert_eq!(tai_minus_utc(2010, 7, 24, 0.0).unwrap().tai_minus_utc, 34.0);

    let tai_days =
        (unix_days_from_civil(2010, 7, 24) - satellite_datetime::TAI_EPOCH_UNIX_DAYS) as i128;
    let expected_tt =
        tai_days * NS_PER_DAY + ((11 * 3600 + 19 * 60 + 13) as i128) * NS_PER_SEC + 502_000_000;
    assert_eq!(inst.reading_tt().as_nanos(), expected_tt);

    let back = inst.to_utc().unwrap();
    assert_eq!(back, utc);
}

/// IERS / ERFA `eraDat`: TAI−UTC after the 2016-12-31 leap is 37 s.
#[test]
fn iers_tai_minus_utc_modern_plateau() {
    assert_eq!(
        tai_minus_utc(2016, 12, 31, 0.0).unwrap().tai_minus_utc,
        36.0
    );
    assert_eq!(tai_minus_utc(2017, 1, 1, 0.0).unwrap().tai_minus_utc, 37.0);
    assert_eq!(tai_minus_utc(2023, 6, 1, 0.0).unwrap().tai_minus_utc, 37.0);
}

/// GPS epoch: 1980-01-06 00:00:00 UTC coincides with GPS 0; TAI−UTC = 19 s.
#[test]
fn gps_epoch_utc_matches_tai_offset() {
    let utc = CivilUtc::new(1980, 1, 6, 0, 0, 0, 0).unwrap();
    let inst = utc.to_instant().unwrap();
    assert_eq!(tai_minus_utc(1980, 1, 6, 0.0).unwrap().tai_minus_utc, 19.0);
    assert_eq!(
        inst.reading_gps().as_nanos(),
        inst.as_tai_nanos() - Duration::from_seconds(19).as_nanos()
    );
    let (week, sow) = inst.gps_week_sow().unwrap();
    assert_eq!(week, 0);
    assert!(sow.abs() < 1e-9);
}

/// TT − TAI is exactly 32.184 s at any instant (IAU).
#[test]
fn tt_minus_tai_is_constant() {
    for (y, m, d) in [(1972, 1, 1), (1999, 1, 1), (2000, 1, 1), (2015, 7, 1)] {
        let inst = CivilUtc::new(y, m, d, 12, 0, 0, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert_eq!(
            inst.reading_tt().as_nanos() - inst.reading_tai().as_nanos(),
            Instant::TAI_EPOCH.reading_tt().as_nanos()
        );
    }
}

/// UTC civil round-trip on days that do not end with a leap second.
#[test]
fn utc_roundtrip_selected_dates() {
    let cases = [
        (1972, 1, 1, 0, 0, 0, 0),
        (2000, 1, 1, 12, 0, 0, 0),
        (2006, 1, 15, 0, 0, 0, 1),
        (2012, 6, 29, 23, 59, 59, 0),
        (2015, 7, 1, 0, 0, 0, 0),
    ];
    for (y, mo, d, h, mi, s, ns) in cases {
        let c = CivilUtc::new(y, mo, d, h, mi, s, ns).unwrap();
        let inst = c
            .to_instant()
            .unwrap_or_else(|e| panic!("to_instant {y}-{mo:02}-{d:02}: {e:?}"));
        let back = inst
            .to_utc()
            .unwrap_or_else(|e| panic!("to_utc {y}-{mo:02}-{d:02}: {e:?}"));
        assert_eq!(back, c, "round-trip {y}-{mo:02}-{d:02}");
    }
}

/// Last civil second before the 2012-06-30 leap second (23:59:60).
#[test]
fn utc_roundtrip_before_2012_leap() {
    let c = CivilUtc::new(2012, 6, 30, 23, 59, 59, 0).unwrap();
    let inst = c.to_instant().expect("to_instant");
    let back = inst.to_utc().expect("to_utc");
    assert_eq!(back, c);
}
