#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//! Experimental timescales for spacecraft, the Moon, and the solar system.
//!
//! **Status: 0.1 work-in-progress.** APIs may break. Not flight-qualified.
//! Lunar TCL is origin-only (no IAU periodic series). Time zones are a seven-zone
//! subset, not the IANA tzdb. Coordinated Lunar Time (LTC) is provisional.
//!
//! An instant is not a calendar. The core stores TAI nanoseconds since
//! 1958-01-01 00:00:00 TAI. UTC leap seconds, DST, sols, and lunar clocks are
//! projections of that value. There is no `now()`; inject a clock.
//!
//! ```
//! use satellite_datetime::{Duration, Instant};
//!
//! let t = Instant::TAI_EPOCH
//!     .checked_add(Duration::from_seconds(1))
//!     .unwrap();
//! assert_eq!(t.as_tai_nanos(), 1_000_000_000);
//! ```
//!
//! Enable `--no-default-features` for the satellite (`no_std`, no allocator) profile.
//!
//! After the first crates.io release, API docs are at
//! <https://docs.rs/satellite-datetime>. Locally: `cargo doc --open`.

mod constants;
pub mod duration;
pub mod error;
pub mod instant;
pub mod julian;
pub mod scale;

#[cfg(feature = "bodies")]
#[cfg_attr(docsrs, doc(cfg(feature = "bodies")))]
pub mod bodies;
#[cfg(feature = "ccsds")]
#[cfg_attr(docsrs, doc(cfg(feature = "ccsds")))]
pub mod ccsds;
#[cfg(feature = "earth")]
#[cfg_attr(docsrs, doc(cfg(feature = "earth")))]
pub mod earth;
#[cfg(feature = "gnss")]
#[cfg_attr(docsrs, doc(cfg(feature = "gnss")))]
pub mod gnss;
#[cfg(feature = "lunar")]
#[cfg_attr(docsrs, doc(cfg(feature = "lunar")))]
pub mod lunar;
#[cfg(feature = "mars")]
#[cfg_attr(docsrs, doc(cfg(feature = "mars")))]
pub mod mars;
#[cfg(feature = "tz")]
#[cfg_attr(docsrs, doc(cfg(feature = "tz")))]
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
#[cfg_attr(docsrs, doc(cfg(feature = "earth")))]
pub use earth::{format_rfc3339, parse_rfc3339, CivilUtc};
