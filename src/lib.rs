#![cfg_attr(not(feature = "std"), no_std)]
//! Satellite-clean timescales: instants are TAI; calendars, DST, sols, and lunar
//! clocks are projections. See the crate README for the timescale glossary.

mod constants;
pub mod duration;
pub mod error;
pub mod instant;
pub mod julian;
pub mod scale;

#[cfg(feature = "bodies")]
pub mod bodies;
#[cfg(feature = "ccsds")]
pub mod ccsds;
#[cfg(feature = "earth")]
pub mod earth;
#[cfg(feature = "gnss")]
pub mod gnss;
#[cfg(feature = "lunar")]
pub mod lunar;
#[cfg(feature = "mars")]
pub mod mars;
#[cfg(feature = "tz")]
pub mod tz;

pub use constants::{
    BDT_MINUS_TAI, GPS_MINUS_TAI, IAU_1977_JD, J2000_JD, LTC_DEFINITION_STATUS,
    LUNAR_SURFACE_MINUS_TT_PER_DAY, L_B, L_G, NS_PER_DAY, NS_PER_SEC, TAI_1977_NS, TAI_EPOCH_JD,
    TAI_EPOCH_MJD, TAI_EPOCH_UNIX_DAYS,
};
pub use duration::Duration;
pub use error::{Error, Result};
pub use instant::Instant;
pub use julian::{civil_from_unix_days, unix_days_from_civil, JulianDate};
pub use scale::{Bdt, Gps, Gst, Ltc, Reading, Scale, Tai, Tcb, Tcg, Tcl, Tdb, Tt};

#[cfg(feature = "earth")]
pub use earth::{format_rfc3339, parse_rfc3339, CivilUtc};
