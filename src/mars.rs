//! Mars Sol Date and Coordinated Mars Time (Allison & McEwen / NASA GISS Mars24).

use crate::instant::Instant;

/// Julian Date TT of the MSD epoch (1873-12-29 12:00-class definition).
pub const MSD_EPOCH_JD_TT: f64 = 2_405_522.002_877_9;
/// Mean Mars sol / Earth day (terrestrial days per sol).
pub const EARTH_DAYS_PER_SOL: f64 = 1.027_491_251_7;

/// Mars Sol Date (running sol count) and mean solar time at the prime meridian.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarsTime {
    /// Sequential sol number (MSD).
    pub msd: f64,
    /// Mean solar time at 0° longitude, hours in `[0, 24)`.
    pub mtc_hours: f64,
}

impl Instant {
    /// MSD and MTC from this instant's TT Julian Date.
    ///
    /// MTC is a UT1 analog (mean solar time), **not** a leap-second UTC analog.
    /// Accuracy is limited by `f64` JD (~50 µs) plus the published sol-length model.
    pub fn mars_time(self) -> MarsTime {
        let jd_tt = self.julian_tt().as_f64();
        let msd = (jd_tt - MSD_EPOCH_JD_TT) / EARTH_DAYS_PER_SOL;
        let frac = msd.rem_euclid(1.0);
        MarsTime {
            msd,
            mtc_hours: frac * 24.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::NS_PER_DAY;
    use crate::duration::Duration;
    use crate::julian::unix_days_from_civil;
    use crate::TAI_EPOCH_UNIX_DAYS;

    #[test]
    fn irfpy_sample_2019() {
        // jd_tt = 2458539.1321896296 → MSD ≈ 51598.61869774 (irfpy/NASA formula)
        let jd_tt = 2_458_539.132_189_629_6;
        let tai_jd = jd_tt - Duration::TT_MINUS_TAI.as_seconds_f64() / 86_400.0;
        let days = tai_jd - crate::TAI_EPOCH_JD;
        let ns = (days * NS_PER_DAY as f64).round() as i128;
        let inst = Instant::from_tai_nanos(ns);
        let mt = inst.mars_time();
        assert!((mt.msd - 51_598.618_697_74).abs() < 1e-6, "msd={}", mt.msd);
        let mtc_h = (51_598.618_697_74_f64).fract() * 24.0;
        assert!((mt.mtc_hours - mtc_h).abs() < 1e-4);
    }

    #[test]
    fn mtc_is_fraction_of_msd() {
        let _ = unix_days_from_civil(2000, 1, 6) - TAI_EPOCH_UNIX_DAYS;
        let inst = Instant::IAU_1977_TAI;
        let mt = inst.mars_time();
        let frac = mt.msd.rem_euclid(1.0);
        assert!((mt.mtc_hours - frac * 24.0).abs() < 1e-12);
    }
}
