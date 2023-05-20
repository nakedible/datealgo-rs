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
//! 
//! # Benchmarks
//! 
//! Results on GitHub Codespaces default VM:
//! 
//! | x                      | datealgo  | hinnant   | httpdate  | humantime | time      | chrono    |
//! | ---------------------- | --------- | --------- | --------- | --------- | --------- | --------- |
//! | rd_to_date             | 5.0 ns    | 9.6 ns    | 12.4 ns   | 12.3 ns   | 23.6 ns   | 10.1 ns   |
//! | date_to_rd             | 3.1 ns    | 3.9 ns    | 4.2 ns    | 3.8 ns    | 18.5 ns   | 8.6 ns    |
//! | systemtime_to_datetime | 16.1 ns   |           | 27.0 ns   | 26.8 ns   | 51.1 ns   | 216.8 ns  |
//! | datetime_to_systemtime | 6.2 ns    |           | 10.9 ns   | 10.1 ns   | 46.1 ns   | 47.5 ns   |
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

pub const SECONDS_MIN: i64 = RD_MIN as i64 * SECS_IN_DAY;
pub const SECONDS_MAX: i64 = RD_MAX as i64 * SECS_IN_DAY;

pub const MONTH_MIN: u32 = 1;
pub const MONTH_MAX: u32 = 12;
pub const JANUARY: u32 = 1;
pub const FEBRUARY: u32 = 2;
pub const MARCH: u32 = 3;
pub const APRIL: u32 = 4;
pub const MAY: u32 = 5;
pub const JUNE: u32 = 6;
pub const JULY: u32 = 7;
pub const AUGUST: u32 = 8;
pub const SEPTEMBER: u32 = 9;
pub const OCTOBER: u32 = 10;
pub const NOVEMBER: u32 = 11;
pub const DECEMBER: u32 = 12;
pub const DAY_MIN: u32 = 1;
pub const DAY_MAX: u32 = 31;
pub const WEEKDAY_MIN: u32 = 1;
pub const WEEKDAY_MAX: u32 = 7;
pub const MONDAY: u32 = 1;
pub const TUESDAY: u32 = 2;
pub const WEDNESDAY: u32 = 3;
pub const THURSDAY: u32 = 4;
pub const FRIDAY: u32 = 5;
pub const SATURDAY: u32 = 6;
pub const SUNDAY: u32 = 7;

pub const RD_I16_MIN: i32 = date_to_rd((i16::MIN as i32, 1, 1));
pub const RD_I16_MAX: i32 = date_to_rd((i16::MAX as i32, 12, 31));
pub const SECS_MIN: i64 = RD_MIN as i64 * SECS_IN_DAY;
pub const SECS_MAX: i64 = RD_MAX as i64 * SECS_IN_DAY + SECS_IN_DAY - 1;

pub const HOUR_MIN: u8 = 0;
pub const HOUR_MAX: u8 = 23;
pub const MINUTE_MIN: u8 = 0;
pub const MINUTE_MAX: u8 = 59;
pub const SECOND_MIN: u8 = 0;
pub const SECOND_MAX: u8 = 59;

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
    debug_assert!(m >= MONTH_MIN && m <= MONTH_MAX, "given month is out of range");
    debug_assert!(d >= DAY_MIN && d <= days_in_month(y, m), "given day is out of range");
    let y = y.wrapping_add(YEAR_OFFSET) as u32;
    let jf = (m < 3) as u32;
    let y = y - jf;
    let m = m + 12 * jf;
    let d = d - 1;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = (979 * m - 2919) / 32;
    let n = y + m + d;
    (n as i32).wrapping_sub(DAY_OFFSET)
}

/// Convert Rata Die to day of week
///
/// Given a day counting from Unix epoch (January 1st, 1970) returns the day of
/// week. Day of week is given as `u32` number between 1 and 7, with `1` meaning
/// Monday and `7` meaning Sunday. Argument must be between `RD_MIN` and
/// `RD_MAX` inclusive. Bounds are checked using `debug_assert` only, so that
/// the checks are not present in release builds, similar to integer overflow
/// checks.
///
/// # Examples
///
/// ```
/// use datealgo::{date_to_rd, rd_to_weekday};
///
/// assert_eq!(rd_to_weekday(date_to_rd((2023, 5, 12))), 5);
/// assert_eq!(rd_to_weekday(date_to_rd((1970, 1, 1))), 4);
/// assert_eq!(rd_to_weekday(date_to_rd((2023, 1, 1))), 7);
/// ```
///
/// If you wish to instead have a value from `0` to `6` with `0` signifying a
/// Sunday, it is easiest to just add `% 7`:
///
/// ```
/// use datealgo::{date_to_rd, rd_to_weekday};
///
/// assert_eq!(rd_to_weekday(date_to_rd((2023, 1, 1))) % 7, 0);
/// assert_eq!(rd_to_weekday(date_to_rd((2023, 5, 12))) % 7, 5);
/// ```
///
/// # Algorithm
///
/// Algorithm is a simple modulus with offset, but argument is first converted
/// to unsigned for performance.
#[inline]
pub const fn rd_to_weekday(n: i32) -> u32 {
    debug_assert!(n >= RD_MIN && n <= RD_MAX, "given rata die is out of range");
    (n.wrapping_add(DAY_OFFSET) as u32 + 2) % 7 + 1
}

/// Convert Gregorian date to day of week
///
/// Given a `(year, month, day)` tuple returns the day of week. Day of week is
/// given as `u32` number between 1 and 7, with `1` meaning Monday and `7`
/// meaning Sunday. Year must be between `YEAR_MIN` and `YEAR_MAX` inclusive.
/// Bounds are checked using `debug_assert` only, so that the checks are not
/// present in release builds, similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::{date_to_weekday};
///
/// assert_eq!(date_to_weekday((2023, 5, 12)), 5);
/// assert_eq!(date_to_weekday((1970, 1, 1)), 4);
/// assert_eq!(date_to_weekday((2023, 1, 1)), 7);
/// ```
///
/// If you wish to instead have a value from `0` to `6` with `0` signifying a
/// Sunday, it is easiest to just add `% 7`:
///
/// ```
/// use datealgo::{date_to_weekday};
///
/// assert_eq!(date_to_weekday((2023, 1, 1)) % 7, 0);
/// assert_eq!(date_to_weekday((2023, 5, 12)) % 7, 5);
/// ```
///
/// # Algorithm
///
/// Algorithm currently used is Sakamoto's method. It is very marginally faster
/// than converting first to Rata Die and determining the day of week from that.
/// If Rata Die value is available, always prefer converting from that.
#[inline]
pub const fn date_to_weekday((y, m, d): (i32, u32, u32)) -> u32 {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    debug_assert!(m >= MONTH_MIN && m <= MONTH_MAX, "given month is out of range");
    debug_assert!(d >= DAY_MIN && d <= days_in_month(y, m), "given day is out of range"); // FIXME
    let y = y.wrapping_add(YEAR_OFFSET) as u32 - (m < 3) as u32;
    let t = [6u8, 2, 1, 4, 6, 2, 4, 0, 3, 5, 1, 3];
    let mut idx = m.wrapping_sub(1) as usize;
    if idx > 11 { idx = 0; } // ensure no bounds check
    (y + y / 4 - y / 100 + y / 400 + t[idx] as u32 + d) % 7 + 1
}

/// Split total seconds to days, hours, minutes and seconds
/// 
/// Given seconds counting from Unix epoch (January 1st, 1970) returns a `(days,
/// hours, minutes, seconds)` tuple. Argument must be between `SECONDS_MIN` and
/// `SECONDS_MAX` inclusive. Bounds are checked using `debug_assert` only, so
/// that the checks are not present in release builds, similar to integer
/// overflow checks.
/// 
/// # Examples
/// 
/// ```
/// use datealgo::{secs_to_dhms, date_to_rd};
/// 
/// assert_eq!(secs_to_dhms(0), (0, 0, 0, 0));
/// assert_eq!(secs_to_dhms(86400), (1, 0, 0, 0));
/// assert_eq!(secs_to_dhms(86399), (0, 23, 59, 59));
/// assert_eq!(secs_to_dhms(-1), (-1, 23, 59, 59));
/// assert_eq!(secs_to_dhms(1684574678), (date_to_rd((2023, 5, 20)), 9, 24, 38));
/// ```
/// 
/// # Algorithm
/// 
/// Algorithm is simple modulus on unsigned values.
#[inline]
pub const fn secs_to_dhms(secs: i64) -> (i32, u8, u8, u8) {
    debug_assert!(secs >= SECONDS_MIN && secs <= SECONDS_MAX, "given seconds value is out of range");
    let secs = secs.wrapping_add(SECS_OFFSET) as u64;
    let days = (secs / SECS_IN_DAY as u64) as u32;
    let secs = (secs % SECS_IN_DAY as u64) as u32;
    let ss = secs % 60;
    let mm = secs / 60 % 60;
    let hh = secs / 3600;
    let days = (days as i32).wrapping_sub(DAY_OFFSET);
    (days, hh as u8, mm as u8, ss as u8)
}

/// Combine days, hours, minutes and seconds to total seconds
#[inline]
pub const fn dhms_to_secs((d, h, m, s): (i32, u8, u8, u8)) -> i64 {
    debug_assert!(d >= RD_MIN && d <= RD_MAX, "given rata die is out of range");
    debug_assert!(h >= HOUR_MIN && h <= HOUR_MAX, "given hour is out of range");
    debug_assert!(m >= MINUTE_MIN && m <= MINUTE_MAX, "given minute is out of range");
    debug_assert!(s >= SECOND_MIN && s <= SECOND_MAX, "given second is out of range");
    d as i64 * SECS_IN_DAY + h as i64 * 3600 + m as i64 * 60 + s as i64
}

#[inline]
pub const fn secs_to_datetime(secs: i64) -> (i32, u32, u32, u8, u8, u8) {
    debug_assert!(secs >= SECS_MIN && secs <= SECS_MAX, "given seconds is out of range");
    let (days, hh, mm, ss) = secs_to_dhms(secs);
    let (y, m, s) = rd_to_date(days);
    (y, m, s, hh, mm, ss)
}

#[inline]
pub const fn datetime_to_secs((y, m, d, hh, mm, ss): (i32, u32, u32, u8, u8, u8)) -> i64 {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    debug_assert!(m >= MONTH_MIN && m <= MONTH_MAX, "given month is out of range");
    debug_assert!(d >= DAY_MIN && d <= days_in_month(y, m), "given day is out of range");
    debug_assert!(hh >= HOUR_MIN && hh <= HOUR_MAX, "given hour is out of range");
    debug_assert!(mm >= MINUTE_MIN && mm <= MINUTE_MAX, "given minute is out of range");
    debug_assert!(ss >= SECOND_MIN && ss <= SECOND_MAX, "given second is out of range");
    let days = date_to_rd((y, m, d));
    dhms_to_secs((days, hh, mm, ss))
}

/// Determine if the given year is a leap year
/// 
/// # Examples
/// 
/// ```
/// use datealgo::is_leap_year;
/// 
/// assert_eq!(is_leap_year(2023), false);
/// assert_eq!(is_leap_year(2024), true);
/// assert_eq!(is_leap_year(2100), false);
/// assert_eq!(is_leap_year(2400), true);
/// ```
/// 
/// # Algorithm
/// 
/// Simple modulus checks on a year transformed positive.
#[inline]
pub const fn is_leap_year(y: i32) -> bool {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    let y = y.wrapping_add(YEAR_OFFSET) as u32;
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

/// Determine the number of days in the given month in the given year
/// 
/// # Example
/// 
/// ```
/// use datealgo::days_in_month;
/// 
/// assert_eq!(days_in_month(2023, 1), 31);
/// assert_eq!(days_in_month(2023, 2), 28);
/// assert_eq!(days_in_month(2023, 4), 30);
/// assert_eq!(days_in_month(2024, 1), 31);
/// assert_eq!(days_in_month(2024, 2), 29);
/// assert_eq!(days_in_month(2024, 4), 30);
/// ```
/// 
/// # Algorithm
/// 
/// Algorithm is table lookup with leap year checking.
#[inline]
pub const fn days_in_month(y: i32, m: u32) -> u32 {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    debug_assert!(m >= MONTH_MIN && m <= MONTH_MAX, "given month is out of range");
    let mut idx = m.wrapping_sub(1) as usize;
    if idx > 11 { idx = 0; }
    if is_leap_year(y) {
        [31u8, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx] as u32
    } else {
        [31u8, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx] as u32
    }
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
pub fn secs_to_systemtime((secs, nsecs): (i64, u32)) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs)
    } else if nsecs > 0 {
        UNIX_EPOCH - Duration::new((-secs - 1) as u64, 1_000_000_000 - nsecs)
    } else {
        UNIX_EPOCH - Duration::from_secs(-secs as u64)
    }
}

#[cfg(feature = "std")]
#[inline]
pub fn systemtime_to_datetime(st: SystemTime) -> (i32, u32, u32, u8, u8, u8, u32) {
    let (secs, nsecs) = systemtime_to_secs(st).unwrap();
    let (days, hh, mm, ss) = secs_to_dhms(secs);
    let (year, month, day) = rd_to_date(days);
    (year, month, day, hh, mm, ss, nsecs)
}

#[cfg(feature = "std")]
#[inline]
pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u32, u32, u8, u8, u8, u32)) -> SystemTime {
    let days = date_to_rd((y, m, d));
    let secs = dhms_to_secs((days, hh, mm, ss));
    secs_to_systemtime((secs, nsec))
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
    fn test_date_to_rd() {
        assert_eq!(date_to_rd((0, 3, 1)), -719468);
        assert_eq!(date_to_rd((1970, 1, 1)), 0);
        assert_eq!(date_to_rd((i16::MIN as i32, 1, 1)), RD_I16_MIN);
        assert_eq!(date_to_rd((i16::MAX as i32, 12, 31)), RD_I16_MAX);
    }

    #[test]
    fn test_rd_to_date() {
        assert_eq!(rd_to_date(-719468), (0, 3, 1));
        assert_eq!(rd_to_date(0), (1970, 1, 1));
        assert_eq!(rd_to_date(RD_I16_MIN), (i16::MIN as i32, 1, 1));
        assert_eq!(rd_to_date(RD_I16_MAX), (i16::MAX as i32, 12, 31));
        assert_eq!(rd_to_date(RD_I16_MIN - 1), (i16::MIN as i32 - 1, 12, 31));
        assert_eq!(rd_to_date(RD_I16_MAX + 1), (i16::MAX as i32 + 1, 1, 1));
    }

    #[test]
    fn test_rd_to_weekday() {
        assert_eq!(rd_to_weekday(RD_MIN), 1);
        assert_eq!(rd_to_weekday(RD_MAX), 4);
        assert_eq!(rd_to_weekday(-719468), 3);
        assert_eq!(rd_to_weekday(0), 4);
        assert_eq!(rd_to_weekday(1), 5);
        assert_eq!(rd_to_weekday(2), 6);
        assert_eq!(rd_to_weekday(3), 7);
        assert_eq!(rd_to_weekday(4), 1);
        assert_eq!(rd_to_weekday(5), 2);
        assert_eq!(rd_to_weekday(6), 3);
        assert_eq!(rd_to_weekday(19489), 5);
    }

    #[test]
    fn test_date_to_weekday() {
        assert_eq!(date_to_weekday((1970, 1, 1)), 4);
        assert_eq!(date_to_weekday((2023, 1, 1)), 7);
    }
}
