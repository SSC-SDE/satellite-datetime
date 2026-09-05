//! GNSS system times as uniform offsets from TAI.

use crate::constants::{BDT_MINUS_TAI, GPS_MINUS_TAI};
use crate::earth::CivilUtc;
use crate::error::Result;
use crate::instant::Instant;
use crate::scale::{Bdt, Gst, Reading};

impl Instant {
    /// GPS week and seconds-of-week (10-bit week rolls every 1024 weeks; we return the full week count from 1980-01-06).
    pub fn gps_week_sow(self) -> Result<(i32, f64)> {
        let epoch = CivilUtc::new(1980, 1, 6, 0, 0, 0, 0)?.to_instant()?;
        let dt = self.duration_since(epoch)?.as_seconds_f64();
        // GPS does not insert leap seconds; elapsed SI from the GPS epoch equals GPS time.
        let week = (dt / 604_800.0).floor() as i32;
        let sow = dt - f64::from(week) * 604_800.0;
        Ok((week, sow))
    }

    pub fn reading_gst(self) -> Reading<Gst> {
        Reading::new(self.as_tai_nanos() + GPS_MINUS_TAI.as_nanos())
    }

    pub fn reading_bdt(self) -> Reading<Bdt> {
        Reading::new(self.as_tai_nanos() + BDT_MINUS_TAI.as_nanos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earth::tai_minus_utc;

    #[test]
    fn gps_epoch_is_utc_midnight() {
        let epoch = CivilUtc::new(1980, 1, 6, 0, 0, 0, 0)
            .unwrap()
            .to_instant()
            .unwrap();
        assert_eq!(tai_minus_utc(1980, 1, 6, 0.0).unwrap().tai_minus_utc, 19.0);
        let (week, sow) = epoch.gps_week_sow().unwrap();
        assert_eq!(week, 0);
        assert!(sow.abs() < 1e-9);
        assert_eq!(
            epoch.reading_gps().as_nanos(),
            epoch.as_tai_nanos() - 19_000_000_000
        );
    }
}
