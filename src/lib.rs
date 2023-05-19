//! Low-level date algorithms for libraries
//!
//! This library aims to provide the **highest performance algorithms** for date
//! manipulation in an unopinionated way. It is meant to be used by the various
//! date and time libraries which can then provide ergonomic and opinionated
//! interfaces for their users.
//!
//! # Usage
//!
//! xxx
//!
//! # Background
//!
//! There are many date and time libraries for Rust for varying use cases as the
//! standard library doesn't include any utilities for dealing with dates. Most
//! of these libraries contain their own copies of date algorithms, most
//! prominently the conversion from days since an epoch to a Gregorian calendar
//! date (year, month, day). These algorithms have been sourced from various
//! places with various licenses, often translated either by machine or by hand
//! from C algorithms found in different libc variants. The algorithms are
//! usually somewhat optimized for performance, but fall short of fastest
//! algorithms available.
//!
//! # Notes
//!
//! The library does not expose any kind of `Date` or `DateTime` structures, but
//! simply tuples for the necessary values. There is bounds checking via
//! `debug_assert`, which means that it is not present in release builds.
//! Callers are required to do their own bounds checking where ever input
//! require it. Datatypes are selected for performance and utility, rather than
//! what is most natural for the value.
//!
//! Currently the library implements algorithms for the [Proleptic Gregorian
//! Calendar](https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar) which
//! is our current calendar extended backwards indefinitely. The Gregorian
//! calendar defines the average year to be 365.2425 days long by defining every
//! fourth year to be a leap year, unless the year is divisible by 100 and not
//! by 400.
//!
//! The algorithms do not account for leap seconds, as is customary for [Unix
//! time](https://en.wikipedia.org/wiki/Unix_time). Every day is exactly 86400
//! in length, and the calculated times do not adjust for leap seconds between
//! timestamps.
//!
//! We define [Rata Die](https://en.wikipedia.org/wiki/Rata_Die) to be integral
//! day numbers counted from 1st of January, 1979, which is the Unix epoch. We
//! use the abbreviation `rd` to concisely refer to such values. This differs
//! from the epoch originally chosen by Howard Jacobson, but is more convenient
//! for usage.
//!
//! The Rata Die values are represented as `i32` for performance reasons. The
//! needed calculations reduce that to roughly an effective `i30` integer range,
//! which means a usable range of roughly -1,460,000 to 1,460,000 years.
#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Adjustment from UNIX epoch to make calculations use positive integers
///
/// Unit is eras, which is defined to be 400 years, as that is the period of the
/// proleptic Gregorian calendar. Selected to place unix epoch roughly in the
/// center of the value space, can be arbitrary within type limits.
const ERA_OFFSET: i32 = 3670;
/// Every era has 146097 days
const DAYS_IN_ERA: i32 = 146097;
/// Every era has 400 years
const YEARS_IN_ERA: i32 = 400;
/// Number of days from 0000-03-01 to UNIX epoch 1970-01-01
const DAYS_TO_UNIX_EPOCH: i32 = 719468;
/// Offset to be added to given day values
const DAY_OFFSET: i32 = ERA_OFFSET * DAYS_IN_ERA + DAYS_TO_UNIX_EPOCH;
/// Offset to be added to given year values
const YEAR_OFFSET: i32 = ERA_OFFSET * YEARS_IN_ERA;
/// Seconds in a single 24 hour calendar day
const SECS_IN_DAY: i64 = 86400;
const SECS_OFFSET: i64 = DAY_OFFSET as i64 * SECS_IN_DAY;

#[cfg(feature = "std")]
const SECS_OFFSET_DURATION: Duration = Duration::from_secs(SECS_OFFSET as u64);

/// Minimum supported year for conversion
///
/// Years earlier than this are not supported and will likely produce incorrect
/// results.
pub const YEAR_MIN: i32 = -1467999;

/// Maximum supported year for conversion
///
/// Years later than this are not supported and will likely produce incorrect
/// results.
pub const YEAR_MAX: i32 = 1471744;

/// Minimum Rata Die for conversion
///
/// Rata die days earlier than this are not supported and will likely produce incorrect
/// results.
pub const RD_MIN: i32 = date_to_rd((YEAR_MIN, 1, 1));
/// Maximum Rata Die for conversion
///
/// Rata die days later than this are not supported and will likely produce incorrect
/// results.
pub const RD_MAX: i32 = date_to_rd((YEAR_MAX, 12, 31));
pub const RD_I16_MIN: i32 = date_to_rd((i16::MIN as i32, 1, 1));
pub const RD_I16_MAX: i32 = date_to_rd((i16::MAX as i32, 12, 31));
pub const SECS_MIN: i64 = RD_MIN as i64 * SECS_IN_DAY;
pub const SECS_MAX: i64 = RD_MAX as i64 * SECS_IN_DAY + SECS_IN_DAY - 1;

// OPTIMIZATION NOTES:
// - addition and substraction is the same speed regardless of signed or unsigned
// - addition and substraction is the same speed for u32 and u64
// - multiplication and especially division is slower for u64 than u32
// - division is slower for signed than unsigned
// - if the addition of two i32 is positive and fits in u32, wrapping (default)
//   semantics give us the correct results even if the sum is larger than i32::MAX

/// Convert Rata Die to Gregorian date
///
/// Given a day counting from Unix epoch (January 1st, 1970) returns a `(year,
/// month, day)` tuple. Argument must be between `RD_MIN` and `RD_MAX`
/// inclusive. Bounds are checked using `debug_assert` only, so that the checks
/// are not present in release builds, similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::rd_to_date;
///
/// assert_eq!(rd_to_date(-719528), (0, 1, 1));
/// assert_eq!(rd_to_date(0), (1970, 1, 1));
/// assert_eq!(rd_to_date(19489), (2023, 5, 12));
/// assert_eq!(rd_to_date(2932896), (9999, 12, 31));
/// assert_eq!(rd_to_date(46761996), (129999, 12, 31));
/// assert_eq!(rd_to_date(-48200687), (-129999, 1, 1));
/// ```
///
/// # Algorithm
///
/// Algorithm currently used is the Neri-Schneider algorithm using Euclidean
/// Affine Functions:
///
/// > Neri C, Schneider L. "*Euclidean affine functions and their application to
/// > calendar algorithms*". Softw Pract Exper. 2022;1-34. doi:
/// > [10.1002/spe.3172](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172).
#[inline]
pub const fn rd_to_date(n: i32) -> (i32, u32, u32) {
    debug_assert!(n >= RD_MIN && n <= RD_MAX, "given rata die is out of range");
    let n = n.wrapping_add(DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let n = n % 146097 / 4;
    let n = 4 * n + 3;
    let p = 2939745 * n as u64;
    let z = (p / 2u64.pow(32)) as u32;
    let n = (p % 2u64.pow(32)) as u32 / 2939745 / 4;
    let nd = (n >= 306) as u32;
    let y = 100 * c + z + nd;
    let n = 2141 * n + 197913;
    let m = n / 2u32.pow(16);
    let d = n % 2u32.pow(16) / 2141;
    let y = (y as i32).wrapping_sub(YEAR_OFFSET);
    let m = m - 12 * nd;
    let d = d + 1;
    (y, m, d)
}

/// Convert Gregorian date to Rata Die
///
/// Given a `(year, month, day)` tuple returns the days since Unix epoch
/// (January 1st, 1970). Dates before the epoch produce negative values. Year
/// must be between `YEAR_MIN` and `YEAR_MAX`, month must be between `1` and
/// `12` and day must be between `1` and the number of days in the month in
/// question. Bounds are checked using `debug_assert` only, so that the checks
/// are not present in release builds, similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::date_to_rd;
///
/// assert_eq!(date_to_rd((2023, 5, 12)), 19489);
/// assert_eq!(date_to_rd((1970, 1, 1)), 0);
/// assert_eq!(date_to_rd((0, 1, 1)), -719528);
/// assert_eq!(date_to_rd((9999, 12, 31)), 2932896);
/// assert_eq!(date_to_rd((129999, 12, 31)), 46761996);
/// assert_eq!(date_to_rd((-129999, 1, 1)), -48200687);
/// ```
///
/// # Algorithm
///
/// Algorithm currently used is the Neri-Schneider algorithm using Euclidean
/// Affine Functions:
///
/// > Neri C, Schneider L. "*Euclidean affine functions and their application to
/// > calendar algorithms*". Softw Pract Exper. 2022;1-34. doi:
/// > [10.1002/spe.3172](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172).
#[inline]
pub const fn date_to_rd((y, m, d): (i32, u32, u32)) -> i32 {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    debug_assert!(m >= 1 && m <= 12, "given month is out of range");
    debug_assert!(d >= 1 && d <= 31, "given day is out of range"); // FIXME
    let y = y.wrapping_add(YEAR_OFFSET) as u32;
    let jf = (m < 3) as u32;
    let y = y - jf;
    let m = m as u32 + 12 * jf;
    let d = d as u32 - 1;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = (979 * m - 2919) / 32;
    let n = y + m + d;
    (n as i32).wrapping_sub(DAY_OFFSET)
}

#[inline]
pub const fn secs_to_dhms(secs: i64) -> (i32, u8, u8, u8) {
    let secs = secs.wrapping_add(SECS_OFFSET) as u64;
    let days = (secs / SECS_IN_DAY as u64) as u32;
    let secs = (secs % SECS_IN_DAY as u64) as u32;
    let ss = secs % 60;
    let mm = secs / 60 % 60;
    let hh = secs / 3600;
    let days = (days as i32).wrapping_sub(DAY_OFFSET);
    (days, hh as u8, mm as u8, ss as u8)
}

#[cfg(feature = "std")]
#[inline]
pub fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
    match st.duration_since(UNIX_EPOCH) {
        Ok(dur) => {
            let secs = dur.as_secs();
            let nsecs = dur.subsec_nanos();
            //if secs > SECS_MAX as u64 { return None; }
            Some((secs as i64, nsecs))
        }
        Err(err) => {
            let dur = err.duration();
            let mut secs = dur.as_secs();
            let mut nsecs = dur.subsec_nanos();
            if nsecs > 0 {
                secs += 1;
                nsecs = 1_000_000_000 - nsecs;
            }
            //if secs > -SECS_MIN as u64 { return None; }
            Some((-(secs as i64), nsecs))
        }
    }
}

#[cfg(feature = "std")]
#[inline]
pub fn systemtime_to_secs2(st: SystemTime) -> Option<(i64, u32)> {
    let dur = st.duration_since(UNIX_EPOCH - SECS_OFFSET_DURATION).ok()?;
    let secs = dur.as_secs();
    // if secs < (SECS_OFFSET + SECS_MIN) as u64 || secs > (SECS_OFFSET + SECS_MAX) as u64 {
    //     return None;
    // }
    let nsecs = dur.subsec_nanos();
    Some((secs as i64 - SECS_OFFSET, nsecs))
}

#[cfg(feature = "std")]
#[inline]
pub fn from_systemtime(st: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
    let (secs, _nsecs) = systemtime_to_secs(st).unwrap();
    let (days, hh, mm, ss) = secs_to_dhms(secs);
    let (year, month, day) = rd_to_date(days);
    let wday = (3 + days) % 7 + 1;
    (year as i16, month as u8, day as u8, hh, mm, ss, wday as u8)
}

#[cfg(feature = "std")]
#[inline]
pub fn to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
    let days = date_to_rd((y as i32, m as u32, d as u32));
    UNIX_EPOCH + Duration::from_secs(ss as u64 + mm as u64 * 60 + hh as u64 * 60 * 60 + (days as u64) * SECS_IN_DAY as u64)
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    
    use super::*;

    #[test]
    fn test_consts() {
        assert_eq!(RD_I16_MIN, -12687794);
        assert_eq!(RD_I16_MAX, 11248737);
        assert_eq!(RD_MIN, -536895152);
        assert_eq!(RD_MAX, 536824295);
    }

    #[test]
    fn test_date_to_rata() {
        assert_eq!(date_to_rd((0, 3, 1)), -719468);
        assert_eq!(date_to_rd((1970, 1, 1)), 0);
        assert_eq!(date_to_rd((i16::MIN as i32, 1, 1)), RD_I16_MIN);
        assert_eq!(date_to_rd((i16::MAX as i32, 12, 31)), RD_I16_MAX);
    }

    #[test]
    fn test_rata_to_date() {
        assert_eq!(rd_to_date(-719468), (0, 3, 1));
        assert_eq!(rd_to_date(0), (1970, 1, 1));
        assert_eq!(rd_to_date(RD_I16_MIN), (i16::MIN as i32, 1, 1));
        assert_eq!(rd_to_date(RD_I16_MAX), (i16::MAX as i32, 12, 31));
        assert_eq!(rd_to_date(RD_I16_MIN - 1), (i16::MIN as i32 - 1, 12, 31));
        assert_eq!(rd_to_date(RD_I16_MAX + 1), (i16::MAX as i32 + 1, 1, 1));
    }
}
