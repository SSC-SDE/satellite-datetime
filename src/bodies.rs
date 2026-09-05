//! Generic solar-system bodies: IAU WGCCRE prime-meridian rotation (sidereal).
//!
//! Mean solar time on a generic body needs an orbital mean-sun model. Where we
//! only have `W0`/`Wdot`, we expose **sidereal time at the prime meridian**.
//! Mars civil time lives in [`crate::mars`]. Do not invent planetary DST.

use crate::instant::Instant;

/// IAU cartographic rotation for a body (Archinal et al. / WGCCRE).
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub name: &'static str,
    /// Prime meridian at J2000, degrees.
    pub w0_deg: f64,
    /// Rotation rate, degrees per day (IAU `d` = days from J2000 TDB ≈ TT).
    pub wdot_deg_per_day: f64,
}

pub const MERCURY: Body = Body {
    name: "Mercury",
    w0_deg: 329.5469,
    wdot_deg_per_day: 6.138_502_5,
};
pub const VENUS: Body = Body {
    name: "Venus",
    w0_deg: 160.20,
    wdot_deg_per_day: -1.481_368_8,
};
pub const EARTH: Body = Body {
    name: "Earth",
    w0_deg: 190.147,
    wdot_deg_per_day: 360.985_623_5,
};
pub const MOON: Body = Body {
    name: "Moon",
    w0_deg: 38.3213,
    wdot_deg_per_day: 13.176_358_15,
};
pub const MARS: Body = Body {
    name: "Mars",
    w0_deg: 176.630,
    wdot_deg_per_day: 350.891_982_26,
};
pub const JUPITER: Body = Body {
    name: "Jupiter",
    w0_deg: 284.95,
    wdot_deg_per_day: 870.536_000_0,
};
pub const SATURN: Body = Body {
    name: "Saturn",
    w0_deg: 38.90,
    wdot_deg_per_day: 810.793_902_4,
};
pub const URANUS: Body = Body {
    name: "Uranus",
    w0_deg: 203.81,
    wdot_deg_per_day: -501.160_092_8,
};
pub const NEPTUNE: Body = Body {
    name: "Neptune",
    w0_deg: 299.36,
    wdot_deg_per_day: 541.139_775_7,
};

pub const PLANETS: &[Body] = &[
    MERCURY, VENUS, EARTH, MOON, MARS, JUPITER, SATURN, URANUS, NEPTUNE,
];

impl Body {
    /// IAU west-longitude prime meridian angle W in degrees, wrapped to `[0, 360)`.
    pub fn prime_meridian_deg(self, instant: Instant) -> f64 {
        let d = instant.julian_tt().as_f64() - crate::J2000_JD;
        wrap_deg(self.w0_deg + self.wdot_deg_per_day * d)
    }

    /// Sidereal hour angle of the prime meridian in hours `[0, 24)`.
    pub fn sidereal_hours(self, instant: Instant) -> f64 {
        self.prime_meridian_deg(instant) / 15.0
    }
}

fn wrap_deg(mut x: f64) -> f64 {
    x %= 360.0;
    if x < 0.0 {
        x += 360.0;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_and_named_bodies() {
        assert_eq!(PLANETS.len(), 9);
        let t = Instant::IAU_1977_TAI;
        let w = MARS.prime_meridian_deg(t);
        assert!(w >= 0.0 && w < 360.0);
        let h = MARS.sidereal_hours(t);
        assert!(h >= 0.0 && h < 24.0);
    }
}
