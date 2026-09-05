//! SI durations. Never a calendar day, never a leap second.

use crate::error::{Error, Result};
use crate::NS_PER_SEC;

/// A signed span of SI seconds, stored as nanoseconds.
///
/// This is the only duration type in the core. Calendar days, sols, and UTC
/// days that contain leap seconds are *not* durations; convert them explicitly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Duration {
    ns: i128,
}

impl Duration {
    /// Zero length.
    pub const ZERO: Self = Self { ns: 0 };
    /// One SI second.
    pub const SECOND: Self = Self { ns: NS_PER_SEC };
    /// One SI millisecond.
    pub const MILLISECOND: Self = Self { ns: 1_000_000 };
    /// One SI microsecond.
    pub const MICROSECOND: Self = Self { ns: 1_000 };
    /// One SI nanosecond.
    pub const NANOSECOND: Self = Self { ns: 1 };

    /// Exact TT − TAI offset (IAU): 32.184 SI seconds.
    pub const TT_MINUS_TAI: Self = Self { ns: 32_184_000_000 };

    /// Construct from SI nanoseconds.
    pub const fn from_nanos(ns: i128) -> Self {
        Self { ns }
    }

    /// Construct from whole SI seconds.
    pub const fn from_seconds(sec: i64) -> Self {
        Self {
            ns: sec as i128 * NS_PER_SEC,
        }
    }

    /// Construct from whole milliseconds.
    pub const fn from_millis(ms: i64) -> Self {
        Self {
            ns: ms as i128 * 1_000_000,
        }
    }

    /// Construct from a floating SI second count. Rounds to nearest nanosecond.
    ///
    /// Prefer integer constructors for flight code.
    pub fn from_seconds_f64(sec: f64) -> Result<Self> {
        if !sec.is_finite() {
            return Err(Error::Overflow);
        }
        let ns = (sec * 1_000_000_000.0).round();
        if ns < i128::MIN as f64 || ns > i128::MAX as f64 {
            return Err(Error::Overflow);
        }
        Ok(Self { ns: ns as i128 })
    }

    /// SI nanoseconds.
    pub const fn as_nanos(self) -> i128 {
        self.ns
    }

    /// Truncated whole SI seconds toward zero.
    pub const fn as_seconds(self) -> i128 {
        self.ns / NS_PER_SEC
    }

    /// SI seconds as `f64` (sub-nanosecond rounding at large magnitudes).
    pub fn as_seconds_f64(self) -> f64 {
        self.ns as f64 / 1_000_000_000.0
    }

    /// Checked addition.
    pub const fn checked_add(self, other: Self) -> Result<Self> {
        match self.ns.checked_add(other.ns) {
            Some(ns) => Ok(Self { ns }),
            None => Err(Error::Overflow),
        }
    }

    /// Checked subtraction.
    pub const fn checked_sub(self, other: Self) -> Result<Self> {
        match self.ns.checked_sub(other.ns) {
            Some(ns) => Ok(Self { ns }),
            None => Err(Error::Overflow),
        }
    }

    /// Checked negation.
    pub const fn checked_neg(self) -> Result<Self> {
        match self.ns.checked_neg() {
            Some(ns) => Ok(Self { ns }),
            None => Err(Error::Overflow),
        }
    }

    /// Multiply by a signed integer.
    pub const fn checked_mul(self, rhs: i128) -> Result<Self> {
        match self.ns.checked_mul(rhs) {
            Some(ns) => Ok(Self { ns }),
            None => Err(Error::Overflow),
        }
    }
}

impl core::ops::Neg for Duration {
    type Output = Result<Self>;
    fn neg(self) -> Self::Output {
        self.checked_neg()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_offset_is_exact() {
        assert_eq!(Duration::TT_MINUS_TAI.as_nanos(), 32_184_000_000);
        assert!((Duration::TT_MINUS_TAI.as_seconds_f64() - 32.184).abs() < 1e-15);
    }

    #[test]
    fn overflow() {
        assert_eq!(
            Duration::from_nanos(i128::MAX).checked_add(Duration::NANOSECOND),
            Err(Error::Overflow)
        );
    }
}
