//! Low-level date algorithms for libraries
//!
//! This library aims to provide the **highest performance algorithms** for date
//! manipulation in an unopinionated way. It is meant to be used by the various
//! date and time libraries which can then provide ergonomic and opinionated
//! interfaces for their users.
//!
//! # Usage
//!
//! The primary contribution of this crate for date libraries are the
//! conversions between a day number from Unix epoch (January 1st, 1970) and a
//! Gregorian date:
//!
//! ```
//! use datealgo::{rd_to_date, date_to_rd};
//!
//! assert_eq!(date_to_rd((1970, 1, 1)), 0);
//! assert_eq!(date_to_rd((2023, 5, 12)), 19489);
//! assert_eq!(rd_to_date(19489), (2023, 5, 12));
//! ```
//!
//! For convenience, there is also converters to and from Unix timestamps:
//!
//! ```
//! use datealgo::{secs_to_datetime, datetime_to_secs};
//!
//! assert_eq!(datetime_to_secs((1970, 1, 1, 0, 0, 0)), 0);
//! assert_eq!(datetime_to_secs((2023, 5, 20, 9, 24, 38)), 1684574678);
//! assert_eq!(secs_to_datetime(1684574678), (2023, 5, 20, 9, 24, 38));
//! ```
//!
//! If the `std` feature is enabled, there are also converters to and from
//! `SystemTime`:
//!
//! ```
//! use datealgo::{systemtime_to_datetime, datetime_to_systemtime};
//! use std::time::{Duration, UNIX_EPOCH};
//!
//! assert_eq!(systemtime_to_datetime(UNIX_EPOCH), Some((1970, 1, 1, 0, 0, 0, 0)));
//! assert_eq!(systemtime_to_datetime(UNIX_EPOCH + Duration::from_secs(1684574678)), Some((2023, 5, 20, 9, 24, 38, 0)));
//! assert_eq!(datetime_to_systemtime((2023, 5, 20, 9, 24, 38, 0)), UNIX_EPOCH + Duration::from_secs(1684574678));
//! ```
//!
//! # Features
//!
//! The crate works in `no_std` environments and has no allocations. Most of the
//! functions also work in constant contexts.
//!
//! - `std` (default): Include `SystemTime` conversions
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
//! simply tuples for the necessary values. Bounds checking is done via
//! `debug_assert` only, which means the methods are guaranteed to not panic in
//! release builds. Callers are required to do their own bounds checking.
//! Datatypes are selected as the smallest that will fit the value.
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
//! day numbers counted from 1st of January, 1970, which is the Unix epoch. We
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
//! Results on GitHub Codespaces 8-core VM:
//!
//! | Function               | [datealgo](https://github.com/nakedible/datealgo-rs) | [hinnant](https://howardhinnant.github.io/date_algorithms.html) | [httpdate](https://github.com/pyfisch/httpdate) | [humantime](https://github.com/tailhook/humantime) | [time](https://github.com/time-rs/time) | [chrono](https://github.com/chronotope/chrono) |
//! | ---------------------- | ------------- | --------- | --------- | --------- | --------- | --------- |
//! | date_to_rd             | **2.5 ns**    | 3.3 ns    | 3.1 ns    | 3.2 ns    | 17.7 ns   | 7.4 ns    |
//! | rd_to_date             | **3.7 ns**    | 7.1 ns    | 11.8 ns   | 11.9 ns   | 18.7 ns   | 8.7 ns    |
//! | datetime_to_systemtime | **6.9 ns**    |           | 10.6 ns   | 9.6 ns    | 58.9 ns   | 50.7 ns   |
//! | systemtime_to_datetime | **14.6 ns**   |           | 20.5 ns   | 20.3 ns   | 57.0 ns   | 228.2 ns  |
//!
//! Some code has been adapted from the libraries to produce comparable
//! benchmarks.
//!
//! # Acknowledgements
//!
//! I do not claim original research on anything that is in this crate.
//!
//! - [Cassio Neri and Lorenz
//!   Schneider](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172):
//!   While searching for best method for date conversion, I stumbled upon a
//!   research paper which explains a novel way to optimize the performance.
//!   These algorithms have been implemented here based on the published
//!   article. This wouldn't be the best performing date conversion library
//!   without their work.
//! - [David Hinnant](https://howardhinnant.github.io/date_algorithms.html):
//!   While searching for "perpetual calendar" algorithms, and having already
//!   started my library, I stumbled upon a very similar idea by David Hinnant.
//!   It remains one of the cleanest and simplest algorithms while still
//!   retaining excellent performance.
//! - [Rich
//!   Felker](https://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c):
//!   The original musl `__time_to_tm` function has spread far and wide and been
//!   translated to many languages, and is still the basis of many of the
//!   standalone implementations littered among the libraries.
//! - [Many authors of newlib
//!   `gmtime_r.c`](https://sourceware.org/git/?p=newlib-cygwin.git;a=blob;f=newlib/libc/time/gmtime_r.c;hb=HEAD):
//!   The newlib implementation has evolved significantly over time and has now
//!   been updated based on the work by David Hinnant.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Adjustment from Unix epoch to make calculations use positive integers
///
/// Unit is eras, which is defined to be 400 years, as that is the period of the
/// proleptic Gregorian calendar. Selected to place unix epoch roughly in the
/// center of the value space, can be arbitrary within type limits.
const ERA_OFFSET: i32 = 3670;
/// Every era has 146097 days
const DAYS_IN_ERA: i32 = 146097;
/// Every era has 400 years
const YEARS_IN_ERA: i32 = 400;
/// Number of days from 0000-03-01 to Unix epoch 1970-01-01
const DAYS_TO_UNIX_EPOCH: i32 = 719468;
/// Offset to be added to given day values
const DAY_OFFSET: i32 = ERA_OFFSET * DAYS_IN_ERA + DAYS_TO_UNIX_EPOCH;
/// Offset to be added to given year values
const YEAR_OFFSET: i32 = ERA_OFFSET * YEARS_IN_ERA;
/// Seconds in a single 24 hour calendar day
const SECS_IN_DAY: i64 = 86400;
/// Offset to be added to given second values
const SECS_OFFSET: i64 = DAY_OFFSET as i64 * SECS_IN_DAY;

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

/// Minimum Rata Die in seconds for conversion
///
/// Rata die seconds earlier than this are not supported and will likely produce incorrect
/// results.
pub const RD_SECONDS_MIN: i64 = RD_MIN as i64 * SECS_IN_DAY;

/// Maximum Rata die in seconds for conversion
///
/// Rata die seconds later than this are not supported and will likely produce incorrect
/// results.
pub const RD_SECONDS_MAX: i64 = RD_MAX as i64 * SECS_IN_DAY + SECS_IN_DAY - 1;

/// Convenience constants, mostly for input validation
///
/// The use of these constants is strictly optional, as this is a low level
/// library and the values are wholly unremarkable.
pub mod consts {
    /// Minimum value for month
    pub const MONTH_MIN: u8 = 1;
    /// Maximum value for month
    pub const MONTH_MAX: u8 = 12;
    /// Minimum value for day of month
    pub const DAY_MIN: u8 = 1;
    /// Maximum value for day of month
    pub const DAY_MAX: u8 = 31;
    /// Minimum value for day of week
    pub const WEEKDAY_MIN: u8 = 1;
    /// Maximum value for day of week
    pub const WEEKDAY_MAX: u8 = 7;
    /// Minimum value for hours
    pub const HOUR_MIN: u8 = 0;
    /// Maximum value for hours
    pub const HOUR_MAX: u8 = 23;
    /// Minimum value for minutes
    pub const MINUTE_MIN: u8 = 0;
    /// Maximum value for minutes
    pub const MINUTE_MAX: u8 = 59;
    /// Minimum value for seconds
    pub const SECOND_MIN: u8 = 0;
    /// Maximum value for seconds
    pub const SECOND_MAX: u8 = 59;
    /// Minimum value for nanoseconds
    pub const NANOSECOND_MIN: u32 = 0;
    /// Maximum value for nanoseconds
    pub const NANOSECOND_MAX: u32 = 999_999_999;

    /// January month value
    pub const JANUARY: u32 = 1;
    /// February month value
    pub const FEBRUARY: u32 = 2;
    /// March month value
    pub const MARCH: u32 = 3;
    /// April month value
    pub const APRIL: u32 = 4;
    /// May month value
    pub const MAY: u32 = 5;
    /// June month value
    pub const JUNE: u32 = 6;
    /// July month value
    pub const JULY: u32 = 7;
    /// August month value
    pub const AUGUST: u32 = 8;
    /// September month value
    pub const SEPTEMBER: u32 = 9;
    /// October month value
    pub const OCTOBER: u32 = 10;
    /// November month value
    pub const NOVEMBER: u32 = 11;
    /// December month value
    pub const DECEMBER: u32 = 12;

    /// Monday day of week value
    pub const MONDAY: u32 = 1;
    /// Tuesday day of week value
    pub const TUESDAY: u32 = 2;
    /// Wednesday day of week value
    pub const WEDNESDAY: u32 = 3;
    /// Thursday day of week value
    pub const THURSDAY: u32 = 4;
    /// Friday day of week value
    pub const FRIDAY: u32 = 5;
    /// Saturday day of week value
    pub const SATURDAY: u32 = 6;
    /// Sunday day of week value
    pub const SUNDAY: u32 = 7;
}

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
/// month, day)` tuple.
///
/// # Panics
///
/// Argument must be between [RD_MIN] and [RD_MAX] inclusive. Bounds are checked
/// using `debug_assert` only, so that the checks are not present in release
/// builds, similar to integer overflow checks.
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
pub const fn rd_to_date(n: i32) -> (i32, u8, u8) {
    debug_assert!(n >= RD_MIN && n <= RD_MAX, "given rata die is out of range");
    let n = n.wrapping_add(DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let r = n % 146097;
    let n = r | 3;
    let p = 2939745 * n as u64;
    let z = (p / 2u64.pow(32)) as u32;
    let n = (p % 2u64.pow(32)) as u32 / 2939745 / 4;
    let nd = (n >= 306) as u32;
    let y = 100 * c + z + nd;
    let n = 2141 * n + 197913;
    let m = n / 2u32.pow(16);
    let d = n % 2u32.pow(16) / 2141;
    let y = (y as i32).wrapping_sub(YEAR_OFFSET);
    let m = if nd == 1 { m - 12 } else { m };
    let d = d + 1;
    (y, m as u8, d as u8)
}

/// Convert Gregorian date to Rata Die
///
/// Given a `(year, month, day)` tuple returns the days since Unix epoch
/// (January 1st, 1970). Dates before the epoch produce negative values.
///
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX]. Month must be between `1`
/// and `12`. Day must be between `1` and the number of days in the month in
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
pub const fn date_to_rd((y, m, d): (i32, u8, u8)) -> i32 {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    debug_assert!(m >= consts::MONTH_MIN && m <= consts::MONTH_MAX, "given month is out of range");
    debug_assert!(d >= consts::DAY_MIN && d <= days_in_month(y, m), "given day is out of range");
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

/// Convert Rata Die to day of week
///
/// Given a day counting from Unix epoch (January 1st, 1970) returns the day of
/// week. Day of week is given as `u32` number between 1 and 7, with `1` meaning
/// Monday and `7` meaning Sunday.
///
/// # Panics
///
/// Argument must be between [RD_MIN] and [RD_MAX] inclusive. Bounds are checked
/// using `debug_assert` only, so that the checks are not present in release
/// builds, similar to integer overflow checks.
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
pub const fn rd_to_weekday(n: i32) -> u8 {
    debug_assert!(n >= RD_MIN && n <= RD_MAX, "given rata die is out of range");
    ((n.wrapping_add(DAY_OFFSET) as u32 + 2) % 7 + 1) as u8
}

/// Convert Gregorian date to day of week
///
/// Given a `(year, month, day)` tuple returns the day of week. Day of week is
/// given as `u32` number between 1 and 7, with `1` meaning Monday and `7`
/// meaning Sunday.
///
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX]. Month must be between `1`
/// and `12`. Day must be between `1` and the number of days in the month in
/// question. Bounds are checked using `debug_assert` only, so that the checks
/// are not present in release builds, similar to integer overflow checks.
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
/// Simply converts date to rata die and then rata die to weekday.
///
#[inline]
pub const fn date_to_weekday((y, m, d): (i32, u8, u8)) -> u8 {
    let rd = date_to_rd((y, m, d));
    rd_to_weekday(rd)
}

/// Split total seconds to days, hours, minutes and seconds
///
/// Given seconds counting from Unix epoch (January 1st, 1970) returns a `(days,
/// hours, minutes, seconds)` tuple.
///
/// # Panics
///
/// Argument must be between [RD_SECONDS_MIN] and [RD_SECONDS_MAX] inclusive.
/// Bounds are checked using `debug_assert` only, so that the checks are not
/// present in release builds, similar to integer overflow checks.
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
    debug_assert!(
        secs >= RD_SECONDS_MIN && secs <= RD_SECONDS_MAX,
        "given seconds value is out of range"
    );
    let secs = if secs > RD_SECONDS_MAX { 0 } else { secs }; // allows compiler to optimize more
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
///
/// Given a `(days, hours, minutes, seconds)` tuple from Unix epoch (January
/// 1st, 1970) returns the total seconds.
///
/// # Panics
///
/// Days must be between [RD_MIN] and [RD_MAX] inclusive. Hours must be between
/// `0` and `23`. Minutes must be between `0` and `59`. Seconds must be between
/// `0` and `59`. Bounds are checked using `debug_assert` only, so that the
/// checks are not present in release builds, similar to integer overflow
/// checks.
///
/// # Examples
///
/// ```
/// use datealgo::{dhms_to_secs, date_to_rd};
///
/// assert_eq!(dhms_to_secs((0, 0, 0, 0)), 0);
/// assert_eq!(dhms_to_secs((1, 0, 0, 0)), 86400);
/// assert_eq!(dhms_to_secs((0, 23, 59, 59)), 86399);
/// assert_eq!(dhms_to_secs((-1, 0, 0, 0)), -86400);
/// assert_eq!(dhms_to_secs((-1, 0, 0, 1)), -86399);
/// assert_eq!(dhms_to_secs((date_to_rd((2023, 5, 20)), 9, 24, 38)), 1684574678)
/// ```
///
/// # Algorithm
///
/// Algorithm is simple multiplication, method provided only as convenience.
#[inline]
pub const fn dhms_to_secs((d, h, m, s): (i32, u8, u8, u8)) -> i64 {
    debug_assert!(d >= RD_MIN && d <= RD_MAX, "given rata die is out of range");
    debug_assert!(h >= consts::HOUR_MIN && h <= consts::HOUR_MAX, "given hour is out of range");
    debug_assert!(m >= consts::MINUTE_MIN && m <= consts::MINUTE_MAX, "given minute is out of range");
    debug_assert!(s >= consts::SECOND_MIN && s <= consts::SECOND_MAX, "given second is out of range");
    d as i64 * SECS_IN_DAY + h as i64 * 3600 + m as i64 * 60 + s as i64
}

/// Convert total seconds to year, month, day, hours, minutes and seconds
///
/// Given seconds counting from Unix epoch (January 1st, 1970) returns a `(year,
/// month, day, hours, minutes, seconds)` tuple.
///
/// # Panics
///
/// Argument must be between [RD_SECONDS_MIN] and [RD_SECONDS_MAX] inclusive.
/// Bounds are checked using `debug_assert` only, so that the checks are not
/// present in release builds, similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::secs_to_datetime;
///
/// assert_eq!(secs_to_datetime(0), (1970, 1, 1, 0, 0, 0));
/// assert_eq!(secs_to_datetime(86400), (1970, 1, 2, 0, 0, 0));
/// assert_eq!(secs_to_datetime(86399), (1970, 1, 1, 23, 59, 59));
/// assert_eq!(secs_to_datetime(-1), (1969, 12, 31, 23, 59, 59));
/// assert_eq!(secs_to_datetime(1684574678), (2023, 5, 20, 9, 24, 38));
/// ```
///
/// # Algorithm
///
/// Combination of existing functions for convenience only.
#[inline]
pub const fn secs_to_datetime(secs: i64) -> (i32, u8, u8, u8, u8, u8) {
    let (days, hh, mm, ss) = secs_to_dhms(secs);
    let (y, m, s) = rd_to_date(days);
    (y, m, s, hh, mm, ss)
}

/// Convert year, month, day, hours, minutes and seconds to total seconds
///
/// Given a `(year, month, day, hours, minutes, seconds)` tuple from Unix epoch
/// (January 1st, 1970) returns the total seconds.
///
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX]. Month must be between `1`
/// and `12`. Day must be between `1` and the number of days in the month in
/// question. Hours must be between `0` and `23`. Minutes must be between `0`
/// and `59`. Seconds must be between `0` and `59`. Bounds are checked using
/// `debug_assert` only, so that the checks are not present in release builds,
/// similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::datetime_to_secs;
///
/// assert_eq!(datetime_to_secs((1970, 1, 1, 0, 0, 0)), 0);
/// assert_eq!(datetime_to_secs((1970, 1, 2, 0, 0, 0)), 86400);
/// assert_eq!(datetime_to_secs((1970, 1, 1, 23, 59, 59)), 86399);
/// assert_eq!(datetime_to_secs((1969, 12, 31, 0, 0, 0)), -86400);
/// assert_eq!(datetime_to_secs((1969, 12, 31, 0, 0, 1)), -86399);
/// assert_eq!(datetime_to_secs((2023, 5, 20, 9, 24, 38)), 1684574678)
/// ```
///
/// # Algorithm
///
/// Algorithm is simple multiplication, method provided only as convenience.
#[inline]
pub const fn datetime_to_secs((y, m, d, hh, mm, ss): (i32, u8, u8, u8, u8, u8)) -> i64 {
    let days = date_to_rd((y, m, d));
    dhms_to_secs((days, hh, mm, ss))
}

/// Determine if the given year is a leap year
///
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX] inclusive. Bounds are checked
/// using `debug_assert` only, so that the checks are not present in release
/// builds, similar to integer overflow checks.
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
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX]. Month must be between `1`
/// and `12`. Bounds are checked using `debug_assert` only, so that the checks
/// are not present in release builds, similar to integer overflow checks.
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
pub const fn days_in_month(y: i32, m: u8) -> u8 {
    debug_assert!(m >= consts::MONTH_MIN && m <= consts::MONTH_MAX, "given month is out of range");
    // ensure compiler doesn't include a bounds check
    if m >= consts::MONTH_MIN && m <= consts::MONTH_MAX {
        let idx = m as usize - 1;
        if is_leap_year(y) {
            [31u8, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx]
        } else {
            [31u8, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx]
        }
    } else {
        0
    }
}

/// Convert [`std::time::SystemTime`] to seconds and nanoseconds
///
/// Given [`std::time::SystemTime`] returns an `Option` of `(seconds,
/// nanoseconds)` tuple from Unix epoch (January 1st, 1970).
///
/// # Errors
///
/// Returns `None` if the time is before [RD_SECONDS_MIN] or after
/// [RD_SECONDS_MAX].
///
/// # Examples
///
/// ```
/// use datealgo::systemtime_to_secs;
/// use std::time::{Duration, UNIX_EPOCH};
///
/// assert_eq!(systemtime_to_secs(UNIX_EPOCH), Some((0, 0)));
/// assert_eq!(systemtime_to_secs(UNIX_EPOCH + Duration::new(1, 0)), Some((1, 0)));
/// assert_eq!(systemtime_to_secs(UNIX_EPOCH + Duration::new(0, 1)), Some((0, 1)));
/// assert_eq!(systemtime_to_secs(UNIX_EPOCH - Duration::new(1, 0)), Some((-1, 0)));
/// assert_eq!(systemtime_to_secs(UNIX_EPOCH - Duration::new(0, 1)), Some((-1, 999_999_999)));
/// ```
///
/// # Algorithm
///
/// Uses `.duration_since(UNIX_EPOCH)` and handles both positive and negative
/// result.
#[cfg(feature = "std")]
#[inline]
pub fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
    match st.duration_since(UNIX_EPOCH) {
        Ok(dur) => {
            let secs = dur.as_secs();
            let nsecs = dur.subsec_nanos();
            if secs > RD_SECONDS_MAX as u64 {
                return None;
            }
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
            if secs > -RD_SECONDS_MIN as u64 {
                return None;
            }
            Some((-(secs as i64), nsecs))
        }
    }
}

/// Convert seconds and nanoseconds to [`std::time::SystemTime`]
///
/// Given a tuple of seconds and nanoseconds counting from Unix epoch (January
/// 1st, 1970) returns [`std::time::SystemTime`].
///
/// # Panics
///
/// Seconds must be between [RD_SECONDS_MIN] and [RD_SECONDS_MAX] inclusive.
/// Nanoseconds must between `0` and `999_999_999`. Bounds are checked using
/// `debug_assert` only, so that the checks are not present in release builds,
/// similar to integer overflow checks.
///
/// # Examples
///
/// ```
/// use datealgo::secs_to_systemtime;
/// use std::time::{Duration, UNIX_EPOCH};
///
/// assert_eq!(secs_to_systemtime((0, 0)), UNIX_EPOCH);
/// assert_eq!(secs_to_systemtime((0, 1)), UNIX_EPOCH + Duration::new(0, 1));
/// assert_eq!(secs_to_systemtime((1, 0)), UNIX_EPOCH + Duration::new(1, 0));
/// assert_eq!(secs_to_systemtime((-1, 999_999_999)), UNIX_EPOCH - Duration::new(0, 1));
/// assert_eq!(secs_to_systemtime((-1, 0)), UNIX_EPOCH - Duration::new(1, 0));
/// assert_eq!(secs_to_systemtime((-2, 999_999_999)), UNIX_EPOCH - Duration::new(1, 1));
/// ```
///
/// # Algorithm
///
/// Combination of existing functions for convenience only.
#[cfg(feature = "std")]
#[inline]
pub fn secs_to_systemtime((secs, nsecs): (i64, u32)) -> SystemTime {
    debug_assert!(secs >= RD_SECONDS_MIN && secs <= RD_SECONDS_MAX, "given seconds is out of range");
    debug_assert!(
        nsecs >= consts::NANOSECOND_MIN && nsecs <= consts::NANOSECOND_MAX,
        "given nanoseconds is out of range"
    );
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs)
    } else if nsecs > 0 {
        UNIX_EPOCH - Duration::new((-secs - 1) as u64, 1_000_000_000 - nsecs)
    } else {
        UNIX_EPOCH - Duration::from_secs(-secs as u64)
    }
}

/// Convert [`std::time::SystemTime`] to year, month, day, hours, minutes,
/// seconds and nanoseconds
///
/// Given [`std::time::SystemTime`] returns an Option of `(year, month, day,
/// hours, minutes, seconds, nanoseconds)` tuple.
///
/// # Errors
///
/// Returns `None` if the time is before [RD_SECONDS_MIN] or after
/// [RD_SECONDS_MAX].
///
/// # Examples
///
/// ```
/// use datealgo::systemtime_to_datetime;
/// use std::time::{Duration, UNIX_EPOCH};
///
/// assert_eq!(systemtime_to_datetime(UNIX_EPOCH), Some((1970, 1, 1, 0, 0, 0, 0)));
/// assert_eq!(systemtime_to_datetime(UNIX_EPOCH + Duration::from_secs(1684574678)), Some((2023, 5, 20, 9, 24, 38, 0)));
/// assert_eq!(systemtime_to_datetime(UNIX_EPOCH - Duration::from_secs(1)), Some((1969, 12, 31, 23, 59, 59, 0)));
/// assert_eq!(systemtime_to_datetime(UNIX_EPOCH - Duration::new(0, 1)), Some((1969, 12, 31, 23, 59, 59, 999_999_999)));
/// ```
///
/// # Algorithm
///
/// Combination of existing functions for convenience only.
#[cfg(feature = "std")]
#[inline]
pub fn systemtime_to_datetime(st: SystemTime) -> Option<(i32, u8, u8, u8, u8, u8, u32)> {
    let (secs, nsecs) = systemtime_to_secs(st)?;
    let (days, hh, mm, ss) = secs_to_dhms(secs);
    let (year, month, day) = rd_to_date(days);
    Some((year, month, day, hh, mm, ss, nsecs))
}

/// Convert year, month, day, hours, minutes, seconds and nanoseconds to
/// [`std::time::SystemTime`]
///
/// Given a `(year, month, day, hours, minutes, seconds, nanoseconds)` tuple
/// from Unix epoch (January 1st, 1970) returns [`std::time::SystemTime`].
///
/// # Panics
///
/// Year must be between [YEAR_MIN] and [YEAR_MAX]. Month must be between `1`
/// and `12`. Day must be between `1` and the number of days in the month in
/// question. Hours must be between `0` and `23`. Minutes must be between `0`
/// and `59`. Seconds must be between `0` and `59`. Nanoseconds must be between
/// `0` and `999_999_999`. Bounds are checked using `debug_assert` only, so that
/// the checks are not present in release builds, similar to integer overflow
/// checks.
///
/// # Examples
///
/// ```
/// use datealgo::datetime_to_systemtime;
/// use std::time::{Duration, UNIX_EPOCH};
///
/// assert_eq!(datetime_to_systemtime((1970, 1, 1, 0, 0, 0, 0)), UNIX_EPOCH);
/// assert_eq!(datetime_to_systemtime((1970, 1, 1, 0, 0, 1, 0)), UNIX_EPOCH + Duration::new(1, 0));
/// assert_eq!(datetime_to_systemtime((2023, 5, 20, 9, 24, 38, 0)), UNIX_EPOCH + Duration::from_secs(1684574678));
/// ```
///
/// # Algorithm
///
/// Combination of existing functions for convenience only.
#[cfg(feature = "std")]
#[inline]
pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u8, u8, u8, u8, u8, u32)) -> SystemTime {
    let days = date_to_rd((y, m, d));
    let secs = dhms_to_secs((days, hh, mm, ss));
    secs_to_systemtime((secs, nsec))
}
