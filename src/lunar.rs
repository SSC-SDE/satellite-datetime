//! Lunar Coordinate Time (IAU 2024 TCL) and provisional Coordinated Lunar Time.

use crate::constants::{
    LTC_DEFINITION_STATUS, LUNAR_SURFACE_MINUS_TT_PER_DAY, NS_PER_DAY, TAI_1977_NS,
};
use crate::duration::Duration;
use crate::instant::Instant;
use crate::scale::{Ltc, Reading, Tcl};

/// IAU 2024 origin: TCB (and TCL) read 1977-01-01 00:00:32.184 at the Moon's center.
/// That event is 1977-01-01 00:00:00 TAI + 32.184 s on TT; we use the TAI instant
/// [`Instant::IAU_1977_TAI`] as the conventional origin event (same as TT/TCG).
pub const TCL_ORIGIN: Instant = Instant::IAU_1977_TAI;

/// Status of the operational LTC realization in this crate version.
pub fn ltc_status() -> &'static str {
    LTC_DEFINITION_STATUS
}

impl Instant {
    /// TCL reading using the IAU shared origin with TCB, plus a **linear** TCB→TCL model.
    ///
    /// Periodic 4-velocity terms of the Moon in the BCRS are **not** applied.
    /// Uncertainty: microseconds to milliseconds depending on span; sufficient for
    /// mission planning, not for picosecond PNT. Surface proper time is
    /// [`Self::lunar_mean_surface_proper`].
    pub fn reading_tcl(self) -> Reading<Tcl> {
        // Linear model: TCL and TCB share origin; leading rate uses TCB reading
        // minus the small Earth–Moon barycentric difference (~GM_E/(c² a_EM)).
        // With periodic terms omitted, TCL reading tracks TCB at the origin convention:
        // TCL(t) = TCB(t) − TCB(t0) + TCL(t0), and TCL(t0)=TCB(t0).
        Reading::new(self.reading_tcb().as_nanos())
    }

    /// Provisional LTC: currently identical to TCL (no extra conventional frequency offset).
    pub fn reading_ltc(self) -> Reading<Ltc> {
        Reading::new(self.reading_tcl().as_nanos())
    }

    /// Mean proper time of a clock at rest on the lunar surface relative to TT.
    ///
    /// Rate: +56.02 µs per terrestrial day (Ashby & Patla 2024; NASA SCaN class figure).
    /// This is **not** TCL. Location-dependent periodic terms are omitted
    /// (uncertainty: ~1 µs class over a day near the equator vs poles).
    pub fn lunar_mean_surface_proper(self) -> Duration {
        let dt = self.as_tai_nanos().saturating_sub(TAI_1977_NS);
        let days = dt as f64 / NS_PER_DAY as f64;
        Duration::from_nanos(
            (days * LUNAR_SURFACE_MINUS_TT_PER_DAY.as_nanos() as f64).round() as i128,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_tcl_matches_tcb() {
        let t0 = Instant::IAU_1977_TAI;
        assert_eq!(t0.reading_tcl().as_nanos(), t0.reading_tcb().as_nanos());
        assert_eq!(t0.reading_ltc().as_nanos(), t0.reading_tcl().as_nanos());
        assert!(ltc_status().contains("provisional"));
    }

    #[test]
    fn lunar_surface_faster_than_tt() {
        let year = t0_plus_days(365);
        let d = year.lunar_mean_surface_proper();
        assert!(d.as_nanos() > 20_000_000); // > 20 ms / year (56µs*365≈20.4ms)
        assert!(d.as_nanos() < 21_000_000);
    }

    fn t0_plus_days(days: i64) -> Instant {
        Instant::IAU_1977_TAI
            .checked_add(Duration::from_nanos(days as i128 * NS_PER_DAY))
            .unwrap()
    }
}
