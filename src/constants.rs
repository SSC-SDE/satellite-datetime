//! IAU and CCSDS constants.

use crate::duration::Duration;

/// Nanoseconds in one SI second.
pub const NS_PER_SEC: i128 = 1_000_000_000;
/// Nanoseconds in a 86400-second nominal day.
pub const NS_PER_DAY: i128 = 86_400 * NS_PER_SEC;

/// Julian Date of 1958-01-01 00:00:00 (TAI epoch, CCSDS).
pub const TAI_EPOCH_JD: f64 = 2_436_204.5;
/// Modified Julian Date of the TAI epoch.
pub const TAI_EPOCH_MJD: i64 = 36_204;
/// Unix day number of the TAI epoch (days since 1970-01-01).
pub const TAI_EPOCH_UNIX_DAYS: i64 = -4_383;

/// 1977-01-01 00:00:00 TAI = JD 2443144.5. IAU origin of TT/TCG/TCB relations.
pub const IAU_1977_JD: f64 = 2_443_144.5;
/// TAI nanoseconds from 1958-01-01 to 1977-01-01 00:00:00 TAI.
/// 19 y including leap days 1960,1964,1968,1972,1976 = 19*365+5 = 6940 days.
pub const TAI_1977_NS: i128 = 6_940 * NS_PER_DAY;

/// J2000.0 TT = 2000-01-01 12:00:00 TT = JD 2451545.0.
pub const J2000_JD: f64 = 2_451_545.0;

/// IAU 2000 L_G: TCG − TT rate.
pub const L_G: f64 = 6.969_290_134e-10;
/// IAU 2006 L_B: TCB − TDB rate.
pub const L_B: f64 = 1.550_519_768e-8;
/// IAU TDB offset constant (seconds).
pub const TDB0: f64 = -6.55e-5;
/// JD_TDB of the TCB/TDB origin (2443144.5003725).
pub const TDB_EPOCH_JD: f64 = 2_443_144.500_372_5;

/// GPS − TAI = −19 s.
pub const GPS_MINUS_TAI: Duration = Duration::from_seconds(-19);
/// BDT − TAI = −33 s.
pub const BDT_MINUS_TAI: Duration = Duration::from_seconds(-33);

/// Mean lunar surface proper time minus TT (Ashby & Patla 2024 class result).
/// Clocks on the Moon run faster than TT by about 56.02 µs per terrestrial day.
pub const LUNAR_SURFACE_MINUS_TT_PER_DAY: Duration = Duration::from_nanos(56_020);

/// IAU 2024: at the defining event, TCL = TCB = 1977-01-01 00:00:32.184.
/// That TCB reading is TT at the 1977 TAI midnight plus 32.184 s = TAI 1977 + 32.184 s.
pub const LTC_DEFINITION_STATUS: &str =
    "provisional-2026: LTC is realized as TCL pending CGPM/BIPM operational offset";
