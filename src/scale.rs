//! Named timescales. Phantom types prevent mixing readings.

use core::marker::PhantomData;

/// A uniform timescale that ticks in SI seconds (no leap seconds).
pub trait Scale: Copy + Clone + core::fmt::Debug + 'static {
    /// IAU / operational abbreviation.
    const NAME: &'static str;
    /// Human-readable definition.
    const DEFINITION: &'static str;
}

/// Marker wrapping a scale-specific clock reading (SI nanoseconds on that scale).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Reading<S: Scale> {
    pub(crate) ns: i128,
    pub(crate) _s: PhantomData<S>,
}

impl<S: Scale> Reading<S> {
    pub(crate) const fn new(ns: i128) -> Self {
        Self {
            ns,
            _s: PhantomData,
        }
    }

    /// Nanoseconds of this timescale since its documented zero (see scale docs).
    pub const fn as_nanos(self) -> i128 {
        self.ns
    }
}

/// International Atomic Time. Continuous SI seconds. Epoch 1958-01-01 00:00:00 TAI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tai;
impl Scale for Tai {
    const NAME: &'static str = "TAI";
    const DEFINITION: &'static str =
        "SI seconds on the geoid, no leap seconds; origin 1958-01-01 00:00:00 TAI (CCSDS).";
}

/// Terrestrial Time. TT = TAI + 32.184 s (exact).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tt;
impl Scale for Tt {
    const NAME: &'static str = "TT";
    const DEFINITION: &'static str =
        "Relativistic coordinate time for Earth geoid; TT = TAI + 32.184 s exactly.";
}

/// Geocentric Coordinate Time (IAU).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tcg;
impl Scale for Tcg {
    const NAME: &'static str = "TCG";
    const DEFINITION: &'static str =
        "Coordinate time of GCRS; TCG−TT = L_G × (JD_TT − 2443144.5) × 86400, L_G = 6.969290134e-10.";
}

/// Barycentric Coordinate Time (IAU).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tcb;
impl Scale for Tcb {
    const NAME: &'static str = "TCB";
    const DEFINITION: &'static str =
        "Coordinate time of BCRS; related to TDB by IAU L_B = 1.550519768e-8.";
}

/// Barycentric Dynamical Time (scaled TCB that stays near TT).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tdb;
impl Scale for Tdb {
    const NAME: &'static str = "TDB";
    const DEFINITION: &'static str =
        "Linearly scaled TCB; TDB−TT is periodic (≈1.6 ms amplitude). Ephemeris time in SPICE.";
}

/// Lunar Coordinate Time (IAU 2024, LCRS).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Tcl;
impl Scale for Tcl {
    const NAME: &'static str = "TCL";
    const DEFINITION: &'static str =
        "Coordinate time of the Lunar Celestial Reference System (IAU 2024). Origin when TCB reads 1977-01-01 00:00:32.184 at the Moon's center of mass.";
}

/// Coordinated Lunar Time. Operational scale; definition still being frozen by NASA/BIPM/CGPM.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Ltc;
impl Scale for Ltc {
    const NAME: &'static str = "LTC";
    const DEFINITION: &'static str =
        "Provisional: currently realized as TCL (no extra conventional frequency offset). Update when CGPM/BIPM publish the operational convention.";
}

/// GPS System Time. TAI − 19 s; epoch 1980-01-06 00:00:00 GPS.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Gps;
impl Scale for Gps {
    const NAME: &'static str = "GPS";
    const DEFINITION: &'static str = "TAI − 19 s; epoch 1980-01-06 00:00:00 GPS (= 00:00:00 UTC).";
}

/// Galileo System Time. TAI − 19 s; epoch 1999-08-22.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Gst;
impl Scale for Gst {
    const NAME: &'static str = "GST";
    const DEFINITION: &'static str =
        "Galileo System Time; TAI − 19 s (modulo GST-GPS small offset, treated as 0 here).";
}

/// BeiDou Time. TAI − 33 s; epoch 2006-01-01 00:00:00 BDT.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Bdt;
impl Scale for Bdt {
    const NAME: &'static str = "BDT";
    const DEFINITION: &'static str = "BeiDou Time; TAI − 33 s; epoch 2006-01-01.";
}
