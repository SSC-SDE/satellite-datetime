//! Physical instants stored as TAI nanoseconds since 1958-01-01 00:00:00 TAI.

use crate::constants::{
    GPS_MINUS_TAI, L_B, L_G, NS_PER_DAY, TAI_1977_NS, TAI_EPOCH_JD, TDB0, TDB_EPOCH_JD,
};
use crate::duration::Duration;
use crate::error::{Error, Result};
use crate::scale::{Gps, Reading, Tai, Tcb, Tcg, Tdb, Tt};

/// A unique event in spacetime's time axis, represented as TAI.
///
/// Civil time, DST, leap seconds, sols, and lunar clocks are projections of
/// this value. Clock sources are injected: the core has no `now()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Instant {
    tai_ns: i128,
}

impl Instant {
    /// TAI epoch: 1958-01-01 00:00:00 TAI.
    pub const TAI_EPOCH: Self = Self { tai_ns: 0 };

    /// IAU 1977-01-01 00:00:00 TAI (origin of TT/TCG/TCB linear relations).
    pub const IAU_1977_TAI: Self = Self {
        tai_ns: TAI_1977_NS,
    };

    /// Construct from TAI nanoseconds since [`Self::TAI_EPOCH`].
    pub const fn from_tai_nanos(tai_ns: i128) -> Self {
        Self { tai_ns }
    }

    /// TAI nanoseconds since the CCSDS/TAI epoch.
    pub const fn as_tai_nanos(self) -> i128 {
        self.tai_ns
    }

    /// Construct from a TAI reading (nanoseconds since TAI epoch).
    pub const fn from_tai(reading: Reading<Tai>) -> Self {
        Self { tai_ns: reading.ns }
    }

    /// TAI reading.
    pub const fn reading_tai(self) -> Reading<Tai> {
        Reading::new(self.tai_ns)
    }

    /// TT reading: TAI + 32.184 s.
    pub const fn reading_tt(self) -> Reading<Tt> {
        Reading::new(self.tai_ns + Duration::TT_MINUS_TAI.as_nanos())
    }

    /// Instant whose TT reading is `ns` nanoseconds after 1958-01-01 00:00:00 TT.
    ///
    /// 1958-01-01 00:00:00 TT = 1957-12-31 23:59:27.816 TAI, so this is TAI_ns = ns − 32.184 s.
    pub const fn from_tt_nanos_since_1958_tt(ns: i128) -> Result<Self> {
        match ns.checked_sub(Duration::TT_MINUS_TAI.as_nanos()) {
            Some(tai_ns) => Ok(Self { tai_ns }),
            None => Err(Error::Overflow),
        }
    }

    /// GPS reading: TAI − 19 s (same origin instant expressed on the GPS clock).
    pub const fn reading_gps(self) -> Reading<Gps> {
        Reading::new(self.tai_ns + GPS_MINUS_TAI.as_nanos())
    }

    /// TCG − TT in nanoseconds (IAU linear term only; exact at the 1977 origin).
    pub fn tcg_minus_tt(self) -> Duration {
        let dt_sec = Self::seconds_from_1977_tt(self);
        Duration::from_seconds_f64(L_G * dt_sec).unwrap_or(Duration::ZERO)
    }

    /// TCG reading.
    pub fn reading_tcg(self) -> Reading<Tcg> {
        let d = self.tcg_minus_tt();
        Reading::new(self.reading_tt().as_nanos() + d.as_nanos())
    }

    /// Few-term TDB − TT (seconds). Amplitude ≈ 1.6 ms. Uncertainty: tens of µs
    /// vs full Fairhead–Bretagnon / ERFA `dtdb` (which needs observer location).
    pub fn tdb_minus_tt_seconds(self) -> f64 {
        let jd_tt = self.julian_tt().as_f64();
        let t = (jd_tt - 2_451_545.0) / 36_525.0;
        // Mean anomaly of Earth (degrees → rad), IAU 2000/2006 leading terms.
        let g = (357.528 + 35_999.050_962 * t) * (core::f64::consts::PI / 180.0);
        0.001_657 * libm::sin(g) + 0.000_013_85 * libm::sin(2.0 * g)
    }

    /// TDB reading (TT plus periodic terms).
    pub fn reading_tdb(self) -> Reading<Tdb> {
        let d = Duration::from_seconds_f64(self.tdb_minus_tt_seconds()).unwrap_or(Duration::ZERO);
        Reading::new(self.reading_tt().as_nanos() + d.as_nanos())
    }

    /// TCB − TDB (IAU 2006 linear).
    pub fn tcb_minus_tdb(self) -> Duration {
        let jd_tdb = self.julian_tt().as_f64() + self.tdb_minus_tt_seconds() / 86_400.0;
        let sec = L_B * (jd_tdb - TDB_EPOCH_JD) * 86_400.0 - TDB0;
        Duration::from_seconds_f64(sec).unwrap_or(Duration::ZERO)
    }

    /// TCB reading.
    pub fn reading_tcb(self) -> Reading<Tcb> {
        let ns = self.reading_tdb().as_nanos() + self.tcb_minus_tdb().as_nanos();
        Reading::new(ns)
    }

    /// Add an SI duration.
    pub const fn checked_add(self, d: Duration) -> Result<Self> {
        match self.tai_ns.checked_add(d.as_nanos()) {
            Some(tai_ns) => Ok(Self { tai_ns }),
            None => Err(Error::Overflow),
        }
    }

    /// Subtract an SI duration.
    pub const fn checked_sub(self, d: Duration) -> Result<Self> {
        match self.tai_ns.checked_sub(d.as_nanos()) {
            Some(tai_ns) => Ok(Self { tai_ns }),
            None => Err(Error::Overflow),
        }
    }

    /// SI duration `self - earlier`.
    pub const fn duration_since(self, earlier: Self) -> Result<Duration> {
        match self.tai_ns.checked_sub(earlier.tai_ns) {
            Some(ns) => Ok(Duration::from_nanos(ns)),
            None => Err(Error::Overflow),
        }
    }

    fn seconds_from_1977_tt(self) -> f64 {
        let tt_ns = self.tai_ns + Duration::TT_MINUS_TAI.as_nanos();
        let origin_tt_ns = TAI_1977_NS + Duration::TT_MINUS_TAI.as_nanos();
        (tt_ns - origin_tt_ns) as f64 / 1_000_000_000.0
    }
}

impl Instant {
    /// Days of TAI from epoch as `f64` (for rate formulas).
    pub fn tai_days(self) -> f64 {
        self.tai_ns as f64 / NS_PER_DAY as f64
    }

    /// Julian Date of TAI epoch plus this instant, as `f64`.
    pub fn jd_tai_f64(self) -> f64 {
        TAI_EPOCH_JD + self.tai_days()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_is_constant_offset() {
        let t = Instant::TAI_EPOCH;
        assert_eq!(t.reading_tt().as_nanos(), Duration::TT_MINUS_TAI.as_nanos());
        let later = t.checked_add(Duration::from_seconds(1_000_000)).unwrap();
        assert_eq!(
            later.reading_tt().as_nanos() - later.reading_tai().as_nanos(),
            Duration::TT_MINUS_TAI.as_nanos()
        );
    }

    #[test]
    fn tcg_equals_tt_at_1977() {
        let t = Instant::IAU_1977_TAI;
        let dt = t.tcg_minus_tt().as_nanos();
        assert!(dt.abs() <= 1, "TCG−TT at origin should be 0 ns, got {dt}");
    }

    #[test]
    fn tcg_grows_with_lg() {
        let year = Duration::from_seconds(32_000_000); // ~370 days
        let t = Instant::IAU_1977_TAI.checked_add(year).unwrap();
        let d = t.tcg_minus_tt().as_seconds_f64();
        let expected = L_G * year.as_seconds_f64();
        assert!((d - expected).abs() < 1e-8, "d={d} expected={expected}");
    }

    #[test]
    fn cannot_overflow_silently() {
        assert!(Instant::from_tai_nanos(i128::MAX)
            .checked_add(Duration::NANOSECOND)
            .is_err());
    }
}
