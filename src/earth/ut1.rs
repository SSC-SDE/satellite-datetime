//! Pinned IERS C04 DUT1 (UT1−UTC) and Earth rotation from UT1.
//!
//! **Hot path** (sub-search-loop): [`utc_mjd`], [`dut1_at`], [`era_at_utc`], [`gmst_mean_at_utc`]
//! on an existing [`CivilUtc`], or [`crate::UtcDay`] / [`crate::UtcContext`] for many instants on one UTC date.
//!
//! **Convenience** (runs [`Instant::to_utc`] search): [`dut1`], [`Instant::julian_ut1`],
//! [`Instant::earth_rotation_angle_rad`], [`Instant::gmst_mean_rad`].
//!
//! DUT1 is sampled at 0h UTC on a 5-day knot grid (see [`UT1_TABLE_VERSION`]).
//! Polar motion and the equation of the equinoxes are **not** included.
//!
//! [`crate::bodies::EARTH`] remains the IAU WGCCRE cartographic rotation model;
//! it is **not** IERS UT1.

use super::ut1_table::{FIRST_MJD, KNOTS, KNOT_SPACING, LAST_MJD};

use crate::duration::Duration;
use crate::error::{Error, Result};
use crate::instant::Instant;
use crate::julian::{unix_days_from_civil, JulianDate};

use super::CivilUtc;

/// Version string for the pinned DUT1 table in this crate build.
pub const UT1_TABLE_VERSION: &str = "IERS-EOP-C04-14-IAU2000A224-MJD61045-5day";

/// Result of a DUT1 lookup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ut1Info {
    /// UT1 − UTC in SI seconds (may be negative).
    pub dut1: Duration,
    /// Table version pin.
    pub table: &'static str,
}

// ---------------------------------------------------------------------------
// Hot path: caller already has civil UTC (no `Instant::to_utc` search).
// ---------------------------------------------------------------------------

/// Modified Julian Date of `utc` (continuous through leap seconds).
pub fn utc_mjd(utc: CivilUtc) -> f64 {
    utc_mjd_f64(utc)
}

/// DUT1 at `utc` via the pinned C04 table (no [`Instant::to_utc`]).
pub fn dut1_at(utc: CivilUtc) -> Result<Ut1Info> {
    let dut1_sec = lookup_dut1_seconds(utc_mjd_f64(utc))?;
    Ok(Ut1Info {
        dut1: Duration::from_seconds_f64(dut1_sec)?,
        table: UT1_TABLE_VERSION,
    })
}

/// Julian Date on UT1 from civil UTC (no [`Instant::to_utc`]).
pub fn julian_ut1_at(utc: CivilUtc) -> Result<JulianDate> {
    let mjd_ut1 = utc_mjd_f64(utc) + lookup_dut1_seconds(utc_mjd_f64(utc))? / 86_400.0;
    Ok(julian_from_mjd(mjd_ut1))
}

/// IAU 2000 Earth Rotation Angle at `utc` (radians, `[0, 2π)`).
pub fn era_at_utc(utc: CivilUtc) -> Result<f64> {
    let jd = julian_ut1_at(utc)?.as_f64();
    Ok(era_from_jd_ut1(jd))
}

/// IAU 2006 mean GMST at `utc` (radians, `[0, 2π)`).
pub fn gmst_mean_at_utc(utc: CivilUtc) -> Result<f64> {
    let jd = julian_ut1_at(utc)?.as_f64();
    Ok(gmst_mean_from_jd_ut1(jd))
}

// ---------------------------------------------------------------------------
// Convenience: `Instant` → search loop → civil → hot path internals.
// ---------------------------------------------------------------------------

/// DUT1 (UT1 − UTC) at this instant via the pinned C04 table.
///
/// Convenience wrapper: [`Instant::to_utc`] then [`dut1_at`]. For hot loops on a known
/// UTC day, use [`crate::UtcDay`] + [`dut1_at`] instead.
pub fn dut1(instant: Instant) -> Result<Ut1Info> {
    dut1_at(instant.to_utc()?)
}

impl Instant {
    /// Julian Date on UT1 (two-part).
    ///
    /// Convenience wrapper around [`julian_ut1_at`] after [`Self::to_utc`].
    pub fn julian_ut1(self) -> Result<JulianDate> {
        julian_ut1_at(self.to_utc()?)
    }

    /// IAU 2000 Earth Rotation Angle θ (radians), folded to `[0, 2π)`.
    ///
    /// Convenience wrapper around [`era_at_utc`] after [`Self::to_utc`].
    pub fn earth_rotation_angle_rad(self) -> Result<f64> {
        era_at_utc(self.to_utc()?)
    }

    /// IAU 2006 mean Greenwich sidereal time (radians), folded to `[0, 2π)`.
    ///
    /// Convenience wrapper around [`gmst_mean_at_utc`] after [`Self::to_utc`].
    pub fn gmst_mean_rad(self) -> Result<f64> {
        gmst_mean_at_utc(self.to_utc()?)
    }
}

fn utc_mjd_f64(utc: CivilUtc) -> f64 {
    let unix_days = unix_days_from_civil(utc.year, utc.month, utc.day) as f64;
    let mjd0 = unix_days + 40_587.0;
    let sod = utc.hour as f64 * 3600.0
        + utc.minute as f64 * 60.0
        + utc.second as f64
        + utc.nanosecond as f64 / 1e9;
    mjd0 + sod / 86_400.0
}

fn julian_from_mjd(mjd: f64) -> JulianDate {
    let jd = mjd + 2_400_000.5;
    let d1 = libm::floor(jd);
    JulianDate { d1, d2: jd - d1 }
}

fn lookup_dut1_seconds(mjd: f64) -> Result<f64> {
    if mjd < FIRST_MJD as f64 {
        return Err(Error::Ut1Undefined);
    }
    if mjd > LAST_MJD as f64 {
        return Err(Error::Ut1TableExpired);
    }
    let span = KNOT_SPACING as f64;
    let idx = libm::floor((mjd - FIRST_MJD as f64) / span) as usize;
    let idx = idx.min(KNOTS.len() - 2);
    let (m0, t0) = KNOTS[idx];
    let (m1, t1) = KNOTS[idx + 1];
    let frac = (mjd - m0 as f64) / (m1 - m0) as f64;
    let tenths = t0 as f64 + frac * (t1 - t0) as f64;
    Ok(tenths / 10_000.0)
}

const J2000_JD: f64 = 2_451_545.0;
const TWO_PI: f64 = 2.0 * core::f64::consts::PI;
const ERA0: f64 = 0.7790572732640;
const ERA_RATE: f64 = 1.002_737_811_911_354_6;

fn era_from_jd_ut1(jd_ut1: f64) -> f64 {
    let era = TWO_PI * (ERA0 + ERA_RATE * (jd_ut1 - J2000_JD));
    fold_two_pi(era)
}

/// IAU 2006 mean GMST polynomial (degrees → radians).
fn gmst_mean_from_jd_ut1(jd_ut1: f64) -> f64 {
    let d = jd_ut1 - J2000_JD;
    let t = d / 36_525.0;
    let gmst_deg =
        280.460_618_37 + 360.985_647_366_29 * d + 0.000_387_933 * t * t - t * t * t / 38_710_000.0;
    let gmst_deg = libm::fmod(gmst_deg, 360.0);
    let gmst_deg = if gmst_deg < 0.0 {
        gmst_deg + 360.0
    } else {
        gmst_deg
    };
    gmst_deg * (core::f64::consts::PI / 180.0)
}

fn fold_two_pi(rad: f64) -> f64 {
    let x = libm::fmod(rad, TWO_PI);
    if x < 0.0 {
        x + TWO_PI
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earth::tai_minus_utc;

    #[test]
    fn knot_hit_1972_epoch() {
        let dut1 = lookup_dut1_seconds(41_317.0).unwrap();
        assert!((dut1 - (-0.045_5)).abs() < 1e-4);
    }

    #[test]
    fn linear_mid_interval() {
        let mid = 41_319.5;
        let d0 = lookup_dut1_seconds(41_317.0).unwrap();
        let d1 = lookup_dut1_seconds(41_322.0).unwrap();
        let mid_d = lookup_dut1_seconds(mid).unwrap();
        assert!((mid_d - (d0 + d1) / 2.0).abs() < 1e-7);
    }

    #[test]
    fn pre_1972_errors() {
        let c = CivilUtc::new(1971, 12, 31, 12, 0, 0, 0).unwrap();
        assert_eq!(dut1_at(c), Err(Error::Ut1Undefined));
    }

    #[test]
    fn post_pin_errors() {
        assert_eq!(
            lookup_dut1_seconds(LAST_MJD as f64 + 1.0),
            Err(Error::Ut1TableExpired)
        );
    }

    #[test]
    fn hot_path_matches_convenience() {
        let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
        let inst = c.to_instant().unwrap();
        let d_slow = dut1(inst).unwrap();
        let d_fast = dut1_at(c).unwrap();
        assert_eq!(d_slow, d_fast);
        let era_slow = inst.earth_rotation_angle_rad().unwrap();
        let era_fast = era_at_utc(c).unwrap();
        assert!((era_slow - era_fast).abs() < 1e-12);
        let gmst_slow = inst.gmst_mean_rad().unwrap();
        let gmst_fast = gmst_mean_at_utc(c).unwrap();
        assert!((gmst_slow - gmst_fast).abs() < 1e-12);
    }

    #[test]
    fn era_in_range() {
        let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 318_000_000).unwrap();
        let era = era_at_utc(c).unwrap();
        assert!(era >= 0.0 && era < TWO_PI);
    }

    #[test]
    fn gmst_in_range() {
        let c = CivilUtc::new(2000, 1, 1, 12, 0, 0, 0).unwrap();
        let gmst = gmst_mean_at_utc(c).unwrap();
        assert!(gmst >= 0.0 && gmst < TWO_PI);
    }

    #[test]
    fn j2000_era_near_classical_gmst() {
        let c = CivilUtc::new(2000, 1, 1, 12, 0, 0, 0).unwrap();
        let jd = julian_ut1_at(c).unwrap().as_f64();
        let era = era_from_jd_ut1(jd);
        let expected = TWO_PI * ERA0;
        assert!((era - expected).abs() < 0.05);
    }

    #[test]
    fn dut1_via_instant() {
        let c = CivilUtc::new(2017, 1, 1, 0, 0, 0, 0).unwrap();
        let inst = c.to_instant().unwrap();
        let info = dut1(inst).unwrap();
        assert_eq!(info.table, UT1_TABLE_VERSION);
        assert!(info.dut1.as_seconds_f64().abs() < 1.0);
        assert_eq!(tai_minus_utc(2017, 1, 1, 0.0).unwrap().tai_minus_utc, 37.0);
    }

    #[test]
    fn utc_mjd_matches_julian() {
        let c = CivilUtc::new(2010, 7, 24, 11, 18, 7, 0).unwrap();
        let mjd = utc_mjd(c);
        assert!(mjd > 55_000.0 && mjd < 56_000.0);
    }
}
