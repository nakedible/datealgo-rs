//! The calendar functions the application calls, as a trait, so that the
//! application can run with different implementations of them.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type Date = (i32, u8, u8);
pub type DateTime = (i32, u8, u8, u8, u8, u8);
pub type DateTimeNanos = (i32, u8, u8, u8, u8, u8, u32);
pub type Dhms = (i32, u8, u8, u8);
/// ISO week date: year, week and day of week (Monday is 1)
pub type IsoWeekDate = (i32, u8, u8);

/// The public functions of datealgo, as associated functions.
///
/// `date_to_rd`, `rd_to_date` and `days_in_month` are required. The other
/// functions have defaults built from those and from each other, as a user of
/// a library without them would write them, so that an implementation that
/// only has some of the functions still uses only its own.
pub trait Calendar {
    /// Passed to `black_box` by each event handler, so that the handlers of
    /// each implementation are distinct code that the compiler cannot merge
    /// with another's, at the same cost for all
    const SALT: u64 = 0;

    fn rd_to_date(n: i32) -> Date;

    fn date_to_rd(d: Date) -> i32;

    fn days_in_month(y: i32, m: u8) -> u8;

    fn rd_to_weekday(n: i32) -> u8 {
        ((n as i64 + 3).rem_euclid(7) + 1) as u8
    }

    fn date_to_weekday(d: Date) -> u8 {
        Self::rd_to_weekday(Self::date_to_rd(d))
    }

    fn next_date(d: Date) -> Date {
        Self::rd_to_date(Self::date_to_rd(d) + 1)
    }

    fn prev_date(d: Date) -> Date {
        Self::rd_to_date(Self::date_to_rd(d) - 1)
    }

    fn secs_to_dhms(secs: i64) -> Dhms {
        let s = secs.rem_euclid(86400) as u32;
        (secs.div_euclid(86400) as i32, (s / 3600) as u8, (s / 60 % 60) as u8, (s % 60) as u8)
    }

    fn dhms_to_secs((d, h, m, s): Dhms) -> i64 {
        d as i64 * 86400 + h as i64 * 3600 + m as i64 * 60 + s as i64
    }

    fn secs_to_datetime(secs: i64) -> DateTime {
        let (n, h, mi, s) = Self::secs_to_dhms(secs);
        let (y, m, d) = Self::rd_to_date(n);
        (y, m, d, h, mi, s)
    }

    fn datetime_to_secs((y, m, d, h, mi, s): DateTime) -> i64 {
        Self::dhms_to_secs((Self::date_to_rd((y, m, d)), h, mi, s))
    }

    fn is_leap_year(y: i32) -> bool {
        Self::days_in_month(y, 2) == 29
    }

    fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
        let wd = Self::rd_to_weekday(n);
        let thursday = n - wd as i32 + 4;
        let (y, _, _) = Self::rd_to_date(thursday);
        let week = (thursday - Self::date_to_rd((y, 1, 1))) / 7 + 1;
        (y, week as u8, wd)
    }

    fn isoweekdate_to_rd((y, w, d): IsoWeekDate) -> i32 {
        let jan4 = Self::date_to_rd((y, 1, 4));
        jan4 - Self::rd_to_weekday(jan4) as i32 + 1 + (w as i32 - 1) * 7 + d as i32 - 1
    }

    fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
        Self::rd_to_isoweekdate(Self::date_to_rd(d))
    }

    fn isoweekdate_to_date(iwd: IsoWeekDate) -> Date {
        Self::rd_to_date(Self::isoweekdate_to_rd(iwd))
    }

    fn isoweeks_in_year(y: i32) -> u8 {
        Self::rd_to_isoweekdate(Self::date_to_rd((y, 12, 28))).1
    }

    fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
        match st.duration_since(UNIX_EPOCH) {
            Ok(d) => Some((d.as_secs() as i64, d.subsec_nanos())),
            Err(e) => {
                let d = e.duration();
                let (s, n) = (d.as_secs() as i64, d.subsec_nanos());
                Some(if n == 0 { (-s, 0) } else { (-s - 1, 1_000_000_000 - n) })
            }
        }
    }

    fn secs_to_systemtime((secs, nanos): (i64, u32)) -> Option<SystemTime> {
        if secs >= 0 {
            UNIX_EPOCH.checked_add(Duration::new(secs as u64, nanos))
        } else {
            UNIX_EPOCH
                .checked_sub(Duration::new(secs.unsigned_abs(), 0))?
                .checked_add(Duration::new(0, nanos))
        }
    }

    fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
        let (secs, nanos) = Self::systemtime_to_secs(st)?;
        let (y, m, d, h, mi, s) = Self::secs_to_datetime(secs);
        Some((y, m, d, h, mi, s, nanos))
    }

    fn datetime_to_systemtime((y, m, d, h, mi, s, nanos): DateTimeNanos) -> Option<SystemTime> {
        Self::secs_to_systemtime((Self::datetime_to_secs((y, m, d, h, mi, s)), nanos))
    }
}

/// Implement every function of `Calendar` by calling the function of the same
/// name in the given module
macro_rules! calendar_from_module {
    ($name:ty, $salt:expr, $lib:ident) => {
        impl Calendar for $name {
            const SALT: u64 = $salt;
            fn rd_to_date(n: i32) -> Date {
                $lib::rd_to_date(n)
            }
            fn date_to_rd(d: Date) -> i32 {
                $lib::date_to_rd(d)
            }
            fn days_in_month(y: i32, m: u8) -> u8 {
                $lib::days_in_month(y, m)
            }
            fn rd_to_weekday(n: i32) -> u8 {
                $lib::rd_to_weekday(n)
            }
            fn date_to_weekday(d: Date) -> u8 {
                $lib::date_to_weekday(d)
            }
            fn next_date(d: Date) -> Date {
                $lib::next_date(d)
            }
            fn prev_date(d: Date) -> Date {
                $lib::prev_date(d)
            }
            fn secs_to_dhms(secs: i64) -> Dhms {
                $lib::secs_to_dhms(secs)
            }
            fn dhms_to_secs(dhms: Dhms) -> i64 {
                $lib::dhms_to_secs(dhms)
            }
            fn secs_to_datetime(secs: i64) -> DateTime {
                $lib::secs_to_datetime(secs)
            }
            fn datetime_to_secs(dt: DateTime) -> i64 {
                $lib::datetime_to_secs(dt)
            }
            fn is_leap_year(y: i32) -> bool {
                $lib::is_leap_year(y)
            }
            fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
                $lib::rd_to_isoweekdate(n)
            }
            fn isoweekdate_to_rd(iwd: IsoWeekDate) -> i32 {
                $lib::isoweekdate_to_rd(iwd)
            }
            fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
                $lib::date_to_isoweekdate(d)
            }
            fn isoweekdate_to_date(iwd: IsoWeekDate) -> Date {
                $lib::isoweekdate_to_date(iwd)
            }
            fn isoweeks_in_year(y: i32) -> u8 {
                $lib::isoweeks_in_year(y)
            }
            fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
                $lib::systemtime_to_secs(st)
            }
            fn secs_to_systemtime(sn: (i64, u32)) -> Option<SystemTime> {
                $lib::secs_to_systemtime(sn)
            }
            fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
                $lib::systemtime_to_datetime(st)
            }
            fn datetime_to_systemtime(dt: DateTimeNanos) -> Option<SystemTime> {
                $lib::datetime_to_systemtime(dt)
            }
        }
    };
}

/// The current datealgo
pub struct Datealgo;
calendar_from_module!(Datealgo, 0, datealgo);

/// No calendar work, as a floor for the other implementations. The results are
/// cheap functions of the arguments that stay valid, with months from 1 to 12
/// and days of week cycling through 1 to 7, so that the application behaves
/// normally around them.
pub struct NoCalendar;
impl Calendar for NoCalendar {
    fn rd_to_date(n: i32) -> Date {
        (
            2000 + (n >> 9 & 63),
            (n >> 5 & 7) as u8 + 1 + (n >> 8 & 3) as u8,
            (n & 15) as u8 + 1,
        )
    }
    fn date_to_rd((y, m, d): Date) -> i32 {
        (y << 9) + ((m as i32) << 5) + d as i32
    }
    fn days_in_month(_: i32, m: u8) -> u8 {
        30 | (m & 1)
    }
    fn rd_to_weekday(n: i32) -> u8 {
        ((n & 7) as u8).max(1)
    }
    fn date_to_weekday((_, m, d): Date) -> u8 {
        ((d ^ m) & 7).max(1)
    }
    fn next_date((y, m, d): Date) -> Date {
        if d < 28 {
            (y, m, d + 1)
        } else if m < 12 {
            (y, m + 1, 1)
        } else {
            (y + 1, 1, 1)
        }
    }
    fn prev_date((y, m, d): Date) -> Date {
        if d > 1 {
            (y, m, d - 1)
        } else if m > 1 {
            (y, m - 1, 28)
        } else {
            (y - 1, 12, 28)
        }
    }
    fn secs_to_dhms(secs: i64) -> Dhms {
        (
            (secs >> 17) as i32,
            (secs >> 12 & 15) as u8,
            (secs >> 6 & 31) as u8,
            (secs & 31) as u8,
        )
    }
    fn dhms_to_secs((d, h, m, s): Dhms) -> i64 {
        (d as i64) << 17 | (h as i64) << 12 | (m as i64) << 6 | s as i64
    }
    fn secs_to_datetime(secs: i64) -> DateTime {
        let n = secs as u64;
        (
            2026,
            (n >> 24 & 7) as u8 + 1,
            (n >> 17 & 15) as u8 + 1,
            (n >> 12 & 15) as u8,
            (n >> 6 & 31) as u8,
            (n & 31) as u8,
        )
    }
    fn datetime_to_secs((y, m, d, h, mi, s): DateTime) -> i64 {
        (y as i64) << 30 | (m as i64) << 24 | (d as i64) << 17 | (h as i64) << 12 | (mi as i64) << 6 | s as i64
    }
    fn is_leap_year(y: i32) -> bool {
        y & 3 == 0
    }
    fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
        (2000 + (n >> 9 & 63), (n >> 3 & 31) as u8 + 1, ((n & 7) as u8).max(1))
    }
    fn isoweekdate_to_rd((y, w, d): IsoWeekDate) -> i32 {
        (y << 9) + ((w as i32) << 3) + d as i32
    }
    fn date_to_isoweekdate((y, m, d): Date) -> IsoWeekDate {
        (y, m << 2 | d >> 3, (d & 7).max(1))
    }
    fn isoweekdate_to_date((y, w, d): IsoWeekDate) -> Date {
        (y, (w >> 2 & 7) + 1, (w & 3) * 7 + d)
    }
    fn isoweeks_in_year(y: i32) -> u8 {
        52 | (y & 1) as u8
    }
    fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
        Some(((st > UNIX_EPOCH) as i64, 0))
    }
    fn secs_to_systemtime(_: (i64, u32)) -> Option<SystemTime> {
        Some(UNIX_EPOCH)
    }
    fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
        let n = (st > UNIX_EPOCH) as u8;
        Some((2026, n + 1, n + 1, n, n, n, 0))
    }
    fn datetime_to_systemtime(_: DateTimeNanos) -> Option<SystemTime> {
        Some(UNIX_EPOCH)
    }
}

/// One of several copies of an implementation. Each copy has its own copies of
/// the event handlers, placed at different addresses, and the benchmarks rotate
/// through them, so that how the code of one implementation happens to be
/// placed affects its results less.
pub struct Placed<C, const K: usize>(std::marker::PhantomData<C>);

macro_rules! delegate {
    ($($name:ident($($a:ident: $t:ty),*) -> $r:ty;)*) => {
        impl<C: Calendar, const K: usize> Calendar for Placed<C, K> {
            const SALT: u64 = C::SALT << 8 | K as u64;
            $(fn $name($($a: $t),*) -> $r {
                C::$name($($a),*)
            })*
        }
    };
}
delegate! {
    rd_to_date(n: i32) -> Date;
    date_to_rd(d: Date) -> i32;
    days_in_month(y: i32, m: u8) -> u8;
    rd_to_weekday(n: i32) -> u8;
    date_to_weekday(d: Date) -> u8;
    next_date(d: Date) -> Date;
    prev_date(d: Date) -> Date;
    secs_to_dhms(secs: i64) -> Dhms;
    dhms_to_secs(dhms: Dhms) -> i64;
    secs_to_datetime(secs: i64) -> DateTime;
    datetime_to_secs(dt: DateTime) -> i64;
    is_leap_year(y: i32) -> bool;
    rd_to_isoweekdate(n: i32) -> IsoWeekDate;
    isoweekdate_to_rd(iwd: IsoWeekDate) -> i32;
    date_to_isoweekdate(d: Date) -> IsoWeekDate;
    isoweekdate_to_date(iwd: IsoWeekDate) -> Date;
    isoweeks_in_year(y: i32) -> u8;
    systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)>;
    secs_to_systemtime(sn: (i64, u32)) -> Option<SystemTime>;
    systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos>;
    datetime_to_systemtime(dt: DateTimeNanos) -> Option<SystemTime>;
}
