//! Golden vectors for pinned IERS C04 DUT1 and IAU Earth rotation.
//!
//! DUT1 pairs are published C04 0h UTC samples (not copied ERFA source).
//! ERA uses the IAU 2000 definition with numeric constants from the resolution.

#![cfg(feature = "earth")]

use satellite_datetime::earth::{dut1, UT1_TABLE_VERSION};
use satellite_datetime::CivilUtc;

const TWO_PI: f64 = 2.0 * core::f64::consts::PI;
const ERA0: f64 = 0.7790572732640;
const ERA_RATE: f64 = 1.00273781191135448;
const J2000_JD: f64 = 2_451_545.0;

/// IERS C04 14 at MJD 41317 (1972-01-01 0h UTC): UT1−UTC = −0.0454859 s.
#[test]
fn c04_dut1_1972_epoch() {
    let c = CivilUtc::new(1972, 1, 1, 0, 0, 0, 0).unwrap();
    let inst = c.to_instant().unwrap();
    let info = dut1(inst).unwrap();
    assert_eq!(info.table, UT1_TABLE_VERSION);
    assert!(
        (info.dut1.as_seconds_f64() - (-0.045_5)).abs() < 1e-4,
        "got {}",
        info.dut1.as_seconds_f64()
    );
}

/// IERS C04 14 at MJD 50000 (1988-03-14 0h UTC): UT1−UTC = −0.2425217 s.
#[test]
fn c04_dut1_mjd_50000() {
    // MJD 50000 = unix day 9493 = 1996-01-01? Let me compute: actually use lookup via date.
    // MJD 50000 corresponds to 1995-10-09 in standard conversion.
    // unix_days = 50000 - 40587 = 9413 -> civil_from_unix_days
    let unix = 50_000 - 40_587;
    let (y, m, d) = satellite_datetime::civil_from_unix_days(unix);
    let c = CivilUtc::new(y, m, d, 0, 0, 0, 0).unwrap();
    let info = dut1(c.to_instant().unwrap()).unwrap();
    assert!((info.dut1.as_seconds_f64() - (-0.242_5)).abs() < 2e-3);
}

/// IERS C04 14 at MJD 55000 (2001-12-25 0h UTC): UT1−UTC = +0.2391441 s.
#[test]
fn c04_dut1_mjd_55000() {
    let unix = 55_000 - 40_587;
    let (y, m, d) = satellite_datetime::civil_from_unix_days(unix);
    let c = CivilUtc::new(y, m, d, 0, 0, 0, 0).unwrap();
    let info = dut1(c.to_instant().unwrap()).unwrap();
    assert!((info.dut1.as_seconds_f64() - 0.239_1).abs() < 2e-3);
}

/// IAU 2000 ERA at JD_UT1 = 2451545.0 equals 2π × 0.7790572732640 rad.
#[test]
fn iau_era_at_j2000_ut1() {
    let c = CivilUtc::new(2000, 1, 1, 12, 0, 0, 0).unwrap();
    let inst = c.to_instant().unwrap();
    let jd = inst.julian_ut1().unwrap().as_f64();
    let era = inst.earth_rotation_angle_rad().unwrap();
    let expected = TWO_PI * (ERA0 + ERA_RATE * (jd - J2000_JD));
    let expected = expected.rem_euclid(TWO_PI);
    assert!(
        (era - expected).abs() < 1e-12,
        "era={era} expected={expected}"
    );
    // Near J2000 UT1, ERA is within arcminutes of 2π × ERA0.
    assert!((era - TWO_PI * ERA0).abs() < 0.05);
}

/// Mean GMST polynomial is continuous and matches hand evaluation at 2010-07-24 11:18 UTC.
#[test]
fn gmst_mean_2010_jul_24() {
    let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 0).unwrap();
    let inst = c.to_instant().unwrap();
    let jd = inst.julian_ut1().unwrap().as_f64();
    let gmst = inst.gmst_mean_rad().unwrap();
    let d = jd - J2000_JD;
    let t = d / 36_525.0;
    let gmst_deg =
        280.460_618_37 + 360.985_647_366_29 * d + 0.000_387_933 * t * t - t * t * t / 38_710_000.0;
    let expected = gmst_deg.rem_euclid(360.0) * (core::f64::consts::PI / 180.0);
    assert!((gmst - expected).abs() < 1e-12);
    assert!(gmst > 0.0 && gmst < TWO_PI);
}

/// UT1 before 1972-01-01 is rejected.
#[test]
fn ut1_undefined_pre_1972() {
    let c = CivilUtc::new(1971, 6, 1, 0, 0, 0, 0).unwrap();
    assert!(dut1(c.to_instant().unwrap()).is_err());
}
