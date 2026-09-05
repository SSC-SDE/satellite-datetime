//! Errors that never panic on library paths.

use core::fmt;

/// Recoverable failure from a conversion, parse, or arithmetic operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// `i128` nanosecond arithmetic overflowed.
    Overflow,
    /// Year, month, or day is not a valid Gregorian calendar date.
    InvalidDate,
    /// Hour, minute, second, or nanosecond is out of range (UTC leap: second may be 60).
    InvalidTime,
    /// The instant is before UTC was defined (1960-01-01).
    UtcUndefined,
    /// Leap-second table does not cover this future UTC date with certainty.
    LeapTableExpired,
    /// ISO 8601 / RFC 3339 text could not be parsed.
    Parse,
    /// Buffer was too small to hold the formatted timestamp.
    BufferTooSmall,
    /// CCSDS field length or payload is illegal.
    Codec,
    /// Local civil time falls in a DST spring-forward gap.
    DstGap,
    /// Requested feature data (zone, body) is not in this build.
    Unsupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("nanosecond arithmetic overflow"),
            Self::InvalidDate => f.write_str("invalid calendar date"),
            Self::InvalidTime => f.write_str("invalid clock time"),
            Self::UtcUndefined => f.write_str("UTC is not defined before 1960-01-01"),
            Self::LeapTableExpired => {
                f.write_str("UTC date is beyond the pinned leap-second table")
            }
            Self::Parse => f.write_str("timestamp parse error"),
            Self::BufferTooSmall => f.write_str("output buffer too small"),
            Self::Codec => f.write_str("time-code codec error"),
            Self::DstGap => f.write_str("civil time does not exist (DST gap)"),
            Self::Unsupported => f.write_str("unsupported in this crate feature set"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Crate-wide result type.
pub type Result<T> = core::result::Result<T, Error>;
