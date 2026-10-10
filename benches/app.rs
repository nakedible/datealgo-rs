//! Application benchmark: ranks calendar implementations by how fast a
//! synthetic application runs with each of them. Run with
//! `scripts/app_bench.sh`.
//!
//! The application processes a fixed trace of events, each one thing a general
//! purpose time library is asked to do: format a timestamp, parse one with
//! validation, decode a batch of them, or do date arithmetic. The inputs come
//! from distributions like those of real software, mostly near the present.
//! Each event is handled by a function that is not inlined, with the calendar
//! code inlined into it as at a call site in user code, and between events the
//! rest of the application runs: reads over a working set larger than L2, and
//! code and branches of its own. Each event depends on the result of the
//! previous one, so calls are not overlapped more than in a real program.
//!
//! An implementation is better if the application is faster with it. The
//! `none` row does no calendar work, so the difference to it is the cost of
//! the calendar code in context.
//!
//! The string parsing and formatting around the calendar code is the same for
//! all rows, so library rows compare only the calendar parts of each library.

use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

type Date = (i32, u8, u8);
type DateTime = (i32, u8, u8, u8, u8, u8);

/// The calendar operations of the application. The defaults are composed from
/// the primitives, which default to datealgo. A candidate implementation
/// overrides a primitive, and a library overrides what it provides.
trait Calendar {
    fn rd_to_date(n: i32) -> Date {
        datealgo::rd_to_date(n)
    }

    fn date_to_rd(d: Date) -> i32 {
        datealgo::date_to_rd(d)
    }

    fn days_in_month(y: i32, m: u8) -> u8 {
        datealgo::days_in_month(y, m)
    }

    fn rd_to_weekday(n: i32) -> u8 {
        datealgo::rd_to_weekday(n)
    }

    fn next_date(d: Date) -> Date {
        datealgo::next_date(d)
    }

    /// Unix seconds to date and time of day
    fn to_datetime(secs: i64) -> DateTime {
        let (n, hh, mm, ss) = datealgo::secs_to_dhms(secs);
        let (y, m, d) = Self::rd_to_date(n);
        (y, m, d, hh, mm, ss)
    }

    /// Date and time of day to Unix seconds, if valid
    fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
        if m < 1 || m > 12 || d < 1 || d > Self::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
            return None;
        }
        Some(datealgo::dhms_to_secs((Self::date_to_rd((y, m, d)), hh, mm, ss)))
    }

    /// The date and day of week some days after the date of a timestamp
    fn shift(secs: i64, days: i32) -> (Date, u8) {
        let n = datealgo::secs_to_dhms(secs).0 + days;
        (Self::rd_to_date(n), Self::rd_to_weekday(n))
    }

    /// The date of a timestamp and the six dates after it, and the day of week
    /// of the first
    fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
        let n = datealgo::secs_to_dhms(secs).0;
        out[0] = Self::rd_to_date(n);
        for i in 1..7 {
            out[i] = Self::next_date(out[i - 1]);
        }
        Self::rd_to_weekday(n)
    }
}

/// The current datealgo
struct Datealgo;
impl Calendar for Datealgo {}

mod candidates {
    use super::{Calendar, Date};
    use datealgo::experiments as x;

    pub struct RdTable;
    impl Calendar for RdTable {
        fn rd_to_date(n: i32) -> Date {
            x::rd_to_date_table(n)
        }
    }

    pub struct DateTable;
    impl Calendar for DateTable {
        fn date_to_rd(d: Date) -> i32 {
            x::date_to_rd_table(d)
        }
    }

    pub struct Copy1;
    impl Calendar for Copy1 {}
    pub struct Copy2;
    impl Calendar for Copy2 {}
    pub struct Copy3;
    impl Calendar for Copy3 {}

    pub struct Leap;
    impl Calendar for Leap {
        fn days_in_month(y: i32, m: u8) -> u8 {
            x::days_in_month_h64(y, m)
        }

        fn next_date(d: Date) -> Date {
            x::next_date_h64(d)
        }
    }
}

/// No calendar work, as a floor for the other rows
struct NoCalendar;
impl Calendar for NoCalendar {
    fn to_datetime(secs: i64) -> DateTime {
        let n = secs as u64;
        (2026, (n & 7) as u8 + 1, (n >> 3 & 15) as u8 + 1, (n >> 7 & 15) as u8, (n >> 11 & 31) as u8, (n >> 16 & 31) as u8)
    }

    fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
        Some(y as i64 ^ (m as i64) << 16 ^ (d as i64) << 24 ^ (hh as i64) << 32 ^ (mm as i64) << 40 ^ (ss as i64) << 48)
    }

    fn shift(secs: i64, days: i32) -> (Date, u8) {
        let n = (secs >> 16) as u32 ^ days as u32;
        ((2026, (n & 7) as u8 + 1, (n >> 3 & 15) as u8 + 1), (n >> 7 & 3) as u8 + 1)
    }

    fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
        let n = (secs >> 16) as u32;
        for (i, d) in out.iter_mut().enumerate() {
            *d = (2026, (n & 7) as u8 + 1, (n >> 3 & 15) as u8 + 1 + i as u8);
        }
        (n >> 7 & 3) as u8 + 1
    }
}

mod libs {
    //! Adapters for other libraries, using each library's own types and
    //! operations as its users would.
    use super::{Calendar, Date, DateTime};

    /// Days from 0001-01-01, counted from 1, to 1970-01-01
    const RD_UNIX_EPOCH: i64 = 719_163;

    fn split(secs: i64) -> (i64, u8, u8, u8) {
        let s = secs.rem_euclid(86400) as u32;
        (secs.div_euclid(86400), (s / 3600) as u8, (s / 60 % 60) as u8, (s % 60) as u8)
    }

    pub struct Chrono;
    impl Calendar for Chrono {
        fn to_datetime(secs: i64) -> DateTime {
            use chrono::{Datelike, Timelike};
            let t = chrono::DateTime::from_timestamp(secs, 0).unwrap();
            (t.year(), t.month() as u8, t.day() as u8, t.hour() as u8, t.minute() as u8, t.second() as u8)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            let date = chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32)?;
            Some(date.and_hms_opt(hh as u32, mm as u32, ss as u32)?.and_utc().timestamp())
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            use chrono::Datelike;
            let date = chrono::DateTime::from_timestamp(secs, 0).unwrap().date_naive();
            let d = date.checked_add_signed(chrono::TimeDelta::days(days as i64)).unwrap();
            ((d.year(), d.month() as u8, d.day() as u8), d.weekday().number_from_monday() as u8)
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            use chrono::Datelike;
            let first = chrono::DateTime::from_timestamp(secs, 0).unwrap().date_naive();
            let mut d = first;
            for slot in out.iter_mut() {
                *slot = (d.year(), d.month() as u8, d.day() as u8);
                d = d.succ_opt().unwrap();
            }
            first.weekday().number_from_monday() as u8
        }
    }

    pub struct Time;
    impl Calendar for Time {
        fn to_datetime(secs: i64) -> DateTime {
            let t = time::UtcDateTime::from_unix_timestamp(secs).unwrap();
            let (y, m, d) = t.to_calendar_date();
            let (hh, mm, ss) = t.as_hms();
            (y, m as u8, d, hh, mm, ss)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            let date = time::Date::from_calendar_date(y, time::Month::try_from(m).ok()?, d).ok()?;
            Some(date.with_hms(hh, mm, ss).ok()?.as_utc().unix_timestamp())
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            let date = time::UtcDateTime::from_unix_timestamp(secs).unwrap().date();
            let d = date.checked_add(time::Duration::days(days as i64)).unwrap();
            let (y, m, dd) = d.to_calendar_date();
            ((y, m as u8, dd), d.weekday().number_from_monday())
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            let first = time::UtcDateTime::from_unix_timestamp(secs).unwrap().date();
            let mut d = first;
            for slot in out.iter_mut() {
                let (y, m, dd) = d.to_calendar_date();
                *slot = (y, m as u8, dd);
                d = d.next_day().unwrap();
            }
            first.weekday().number_from_monday()
        }
    }

    /// ICU4X `Date<Iso>`, which has no timestamps, so the time of day is split
    /// off here
    pub struct Icu;
    impl Icu {
        fn date(n: i64) -> icu_calendar::Date<icu_calendar::Iso> {
            icu_calendar::Date::from_rata_die(icu_calendar::types::RataDie::new(n + RD_UNIX_EPOCH), icu_calendar::Iso)
        }

        fn fields(d: &icu_calendar::Date<icu_calendar::Iso>) -> Date {
            (d.year().extended_year(), d.month().ordinal, d.day_of_month().0)
        }
    }
    impl Calendar for Icu {
        fn to_datetime(secs: i64) -> DateTime {
            let (n, hh, mm, ss) = split(secs);
            let (y, m, d) = Self::fields(&Self::date(n));
            (y, m, d, hh, mm, ss)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            if hh > 23 || mm > 59 || ss > 59 {
                return None;
            }
            let date = icu_calendar::Date::try_new_iso(y, m, d).ok()?;
            let n = date.to_rata_die().to_i64_date() - RD_UNIX_EPOCH;
            Some(n * 86400 + hh as i64 * 3600 + mm as i64 * 60 + ss as i64)
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            use icu_calendar::{options::DateAddOptions, types::DateDuration};
            let date = Self::date(split(secs).0);
            let d = date
                .try_added_with_options(DateDuration::for_days(days), DateAddOptions::default())
                .unwrap();
            (Self::fields(&d), d.weekday() as u8)
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            use icu_calendar::{options::DateAddOptions, types::DateDuration};
            let first = Self::date(split(secs).0);
            let mut d = first;
            for slot in out.iter_mut() {
                *slot = Self::fields(&d);
                d = d
                    .try_added_with_options(DateDuration::for_days(1), DateAddOptions::default())
                    .unwrap();
            }
            first.weekday() as u8
        }
    }

    pub struct Fasttime;
    impl Calendar for Fasttime {
        fn to_datetime(secs: i64) -> DateTime {
            let t = fasttime::DateTime::from_unix_timestamp(secs, 0).unwrap();
            (t.date.year, t.date.month, t.date.day, t.time.hour, t.time.minute, t.time.second)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            if hh > 23 || mm > 59 || ss > 59 {
                return None;
            }
            let date = fasttime::Date::from_ymd(y, m, d).ok()?;
            let time = fasttime::Time {
                hour: hh,
                minute: mm,
                second: ss,
                nanosecond: 0,
            };
            Some(fasttime::DateTime::new(date, time).unix_timestamp())
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            let date = fasttime::DateTime::from_unix_timestamp(secs, 0).unwrap().date;
            let d = date.add_days(days as i64).unwrap();
            ((d.year, d.month, d.day), d.weekday().number_from_monday())
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            let first = fasttime::DateTime::from_unix_timestamp(secs, 0).unwrap().date;
            let mut d = first;
            for slot in out.iter_mut() {
                *slot = (d.year, d.month, d.day);
                d = d.add_days(1).unwrap();
            }
            first.weekday().number_from_monday()
        }
    }

    /// hifitime, which works in TAI and applies leap seconds
    pub struct Hifitime;
    impl Hifitime {
        fn epoch(secs: i64) -> hifitime::Epoch {
            hifitime::Epoch::from_unix_duration(hifitime::Duration::from_total_nanoseconds(secs as i128 * 1_000_000_000))
        }
    }
    impl Calendar for Hifitime {
        fn to_datetime(secs: i64) -> DateTime {
            let (y, m, d, hh, mm, ss, _) = Self::epoch(secs).to_gregorian_utc();
            (y, m, d, hh, mm, ss)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            let epoch = hifitime::Epoch::maybe_from_gregorian_utc(y, m, d, hh, mm, ss, 0).ok()?;
            Some((epoch.to_unix_duration().total_nanoseconds() / 1_000_000_000) as i64)
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            let midnight = secs.div_euclid(86400) * 86400;
            let e = Self::epoch(midnight) + hifitime::Unit::Day * days as i64;
            let (y, m, d, ..) = e.to_gregorian_utc();
            ((y, m, d), e.weekday_utc() as u8 + 1)
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            let first = Self::epoch(secs.div_euclid(86400) * 86400);
            let mut e = first;
            for slot in out.iter_mut() {
                let (y, m, d, ..) = e.to_gregorian_utc();
                *slot = (y, m, d);
                e += hifitime::Unit::Day * 1;
            }
            first.weekday_utc() as u8 + 1
        }
    }

    /// temporal_rs, which only provides the primitives, so the rest is
    /// datealgo
    pub struct Temporal;
    impl Calendar for Temporal {
        fn rd_to_date(n: i32) -> Date {
            timezone_provider::utils::neri_schneider::ymd_from_epoch_days(n)
        }

        fn date_to_rd((y, m, d): Date) -> i32 {
            timezone_provider::utils::neri_schneider::epoch_days_from_gregorian_date(y, m, d) as i32
        }

        fn days_in_month(y: i32, m: u8) -> u8 {
            timezone_provider::utils::iso_days_in_month(y, m)
        }
    }
}

// Event handlers, each a call site in the application. Each returns a hash of
// its result.

const DIGITS: &[u8; 200] = b"\
    0001020304050607080910111213141516171819\
    2021222324252627282930313233343536373839\
    4041424344454647484950515253545556575859\
    6061626364656667686970717273747576777879\
    8081828384858687888990919293949596979899";

fn put2(buf: &mut [u8], v: u8) {
    let i = v as usize % 100 * 2;
    buf[0] = DIGITS[i];
    buf[1] = DIGITS[i + 1];
}

fn get2(buf: &[u8]) -> Option<u8> {
    let (a, b) = (buf[0].wrapping_sub(b'0'), buf[1].wrapping_sub(b'0'));
    if a > 9 || b > 9 {
        return None;
    }
    Some(a * 10 + b)
}

fn hash_bytes(buf: &[u8; 20]) -> u64 {
    let a = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let b = u64::from_le_bytes(buf[8..16].try_into().unwrap());
    let c = u32::from_le_bytes(buf[16..20].try_into().unwrap()) as u64;
    a ^ b.rotate_left(21) ^ c.rotate_left(42)
}

fn hash_date((y, m, d): Date) -> u64 {
    (y as u64) << 16 | (m as u64) << 8 | d as u64
}

/// Format a timestamp as RFC 3339, as for a log line or a serialized value
#[inline(never)]
fn format<C: Calendar>(secs: i64) -> u64 {
    let (y, m, d, hh, mm, ss) = C::to_datetime(secs);
    let mut buf = *b"0000-00-00T00:00:00Z";
    put2(&mut buf[0..2], (y / 100) as u8);
    put2(&mut buf[2..4], (y % 100) as u8);
    put2(&mut buf[5..7], m);
    put2(&mut buf[8..10], d);
    put2(&mut buf[11..13], hh);
    put2(&mut buf[14..16], mm);
    put2(&mut buf[17..19], ss);
    hash_bytes(&buf)
}

/// Parse and validate an RFC 3339 timestamp, as from a request or a file
#[inline(never)]
fn parse<C: Calendar>(s: &[u8; 20]) -> u64 {
    let parsed = (|| {
        if s[4] != b'-' || s[7] != b'-' || s[10] != b'T' || s[13] != b':' || s[16] != b':' || s[19] != b'Z' {
            return None;
        }
        let y = get2(&s[0..2])? as i32 * 100 + get2(&s[2..4])? as i32;
        let dt = (y, get2(&s[5..7])?, get2(&s[8..10])?, get2(&s[11..13])?, get2(&s[14..16])?, get2(&s[17..19])?);
        C::from_datetime(dt)
    })();
    match parsed {
        Some(secs) => secs as u64,
        None => 0x5eed,
    }
}

/// Decode a column of timestamps, as from a database
#[inline(never)]
fn decode<C: Calendar>(column: &[i64]) -> u64 {
    let mut h = 0u64;
    for &secs in column {
        let (y, m, d, hh, mm, ss) = C::to_datetime(secs);
        h = h.wrapping_mul(31) ^ hash_date((y, m, d)) ^ (hh as u64) << 40 ^ (mm as u64) << 48 ^ (ss as u64) << 56;
    }
    h
}

/// Find a date some days from a timestamp, as for a due date
#[inline(never)]
fn shift<C: Calendar>(secs: i64, days: i32) -> u64 {
    let (date, wd) = C::shift(secs, days);
    hash_date(date) ^ (wd as u64) << 32
}

/// List the dates of a week from a timestamp, as for a calendar view
#[inline(never)]
fn week<C: Calendar>(secs: i64) -> u64 {
    let mut dates = [(0, 0, 0); 7];
    let wd = C::week(secs, &mut dates);
    dates.iter().fold(wd as u64, |h, &d| h.rotate_left(9) ^ hash_date(d))
}

// The rest of the application

/// Words in the working set of the rest of the application, 4 MiB
const WORKING_SET: usize = 1 << 19;

/// Some of the rest of the application, reading `lines` cache lines from its
/// working set, with a branch on each value read. Each instance has code and
/// branches of its own.
#[inline(never)]
fn other<const K: u64>(mem: &[u64], seed: u64, lines: u32) -> u64 {
    let mut x = seed | 1;
    let mut acc = K;
    for _ in 0..lines {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let v = mem[(x as usize) & (WORKING_SET - 8)];
        if v >> (K % 61) & 1 != 0 {
            acc = acc.wrapping_add(v ^ K);
        } else {
            acc = acc.rotate_left((K % 31) as u32 + 1) ^ v;
        }
    }
    acc
}

macro_rules! others {
    ($($k:literal)*) => { [$(other::<$k>,)*] };
}

type Other = fn(&[u64], u64, u32) -> u64;
const OTHERS: [Other; 64] = others!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

// The trace

const FORMAT: u8 = 0;
const PARSE: u8 = 1;
const DECODE: u8 = 2;
const SHIFT: u8 = 3;
const WEEK: u8 = 4;

/// Probability of each kind of event in percent, in the order of the kinds
const MIX: [u32; 5] = [40, 25, 10, 15, 10];

/// Timestamps in a decoded column
const COLUMN: usize = 8;

/// Cache lines read by the rest of the application between two events, and
/// the probability of each in percent
const OTHER_LINES: [(u16, u32); 4] = [(4, 60), (16, 30), (64, 8), (1024, 2)];

/// Events in the trace, replayed in a loop
const EVENTS: usize = 1 << 18;

/// Events per timed iteration
const CHUNK: usize = 1024;

/// Size of the pools of parsed strings and decoded columns
const POOL: usize = 1 << 16;

#[derive(Clone, Copy)]
struct Event {
    kind: u8,
    other: u8,
    lines: u16,
    days: i32,
    /// A timestamp, or an index to the strings or the column pool
    arg: i64,
}

struct App {
    events: Vec<Event>,
    strings: Vec<[u8; 20]>,
    columns: Vec<i64>,
    mem: Vec<u64>,
}

fn pick<T: Copy>(choices: &[(T, u32)]) -> T {
    let mut r = fastrand::u32(0..choices.iter().map(|c| c.1).sum());
    for &(v, p) in choices {
        if r < p {
            return v;
        }
        r -= p;
    }
    unreachable!()
}

fn secs(dt: DateTime) -> i64 {
    datealgo::datetime_to_secs(dt)
}

impl App {
    fn new() -> App {
        fastrand::seed(7);
        let now = secs((2026, 10, 10, 0, 0, 0));
        let year = 365 * 86400;
        let near_now = || fastrand::i64(now - 2 * year..=now + 2 * year);
        // Timestamps of a value or a request: mostly near the present, some
        // birth dates and some anywhere in 1970-2100.
        let value = || match fastrand::u32(0..10) {
            0..=6 => near_now(),
            7..=8 => secs((fastrand::i32(1940..=2010), 1, 1, 0, 0, 0)) + fastrand::i64(0..365) * 86400,
            _ => fastrand::i64(0..secs((2100, 12, 31, 23, 59, 59))),
        };
        let strings = (0..POOL + 1)
            .map(|_| {
                let (y, m, mut d, hh, mm, ss) = datealgo::secs_to_datetime(value());
                // Some invalid dates, such as the 31st of a 30 day month
                if fastrand::u32(0..50) == 0 {
                    d = datealgo::days_in_month(y, m) + 1;
                }
                let mut s = *b"0000-00-00T00:00:00Z";
                put2(&mut s[0..2], (y / 100) as u8);
                put2(&mut s[2..4], (y % 100) as u8);
                put2(&mut s[5..7], m);
                put2(&mut s[8..10], d);
                put2(&mut s[11..13], hh);
                put2(&mut s[14..16], mm);
                put2(&mut s[17..19], ss);
                s
            })
            .collect();
        // Columns of a database: sorted runs of recent rows, and runs of rows
        // anywhere in 2000-2030.
        let mut columns = Vec::with_capacity(POOL + COLUMN + 1);
        while columns.len() < POOL + COLUMN + 1 {
            if fastrand::bool() {
                let mut t = near_now();
                for _ in 0..64 {
                    t += fastrand::i64(0..3600);
                    columns.push(t);
                }
            } else {
                let (lo, hi) = (secs((2000, 1, 1, 0, 0, 0)), secs((2030, 12, 31, 23, 59, 59)));
                columns.extend((0..64).map(|_| fastrand::i64(lo..=hi)));
            }
        }
        // Log lines: increasing timestamps a few seconds apart
        let mut log = now - 30 * 86400;
        let kinds = [(FORMAT, MIX[0]), (PARSE, MIX[1]), (DECODE, MIX[2]), (SHIFT, MIX[3]), (WEEK, MIX[4])];
        let events = (0..EVENTS)
            .map(|_| {
                let kind = pick(&kinds);
                let (arg, days) = match kind {
                    FORMAT => {
                        log += fastrand::i64(0..=20);
                        (log, 0)
                    }
                    PARSE | DECODE => (fastrand::usize(0..POOL) as i64, 0),
                    SHIFT => (near_now(), if fastrand::bool() { fastrand::i32(1..=30) } else { fastrand::i32(-400..=400) }),
                    _ => (near_now(), 0),
                };
                Event {
                    kind,
                    other: fastrand::u8(0..OTHERS.len() as u8),
                    lines: pick(&OTHER_LINES),
                    days,
                    arg,
                }
            })
            .collect();
        let mem = (0..WORKING_SET).map(|_| fastrand::u64(..)).collect();
        App {
            events,
            strings,
            columns,
            mem,
        }
    }

    /// Process `CHUNK` events from `start`, continuing from the hash `acc` of
    /// the previous results
    fn run<C: Calendar>(&self, start: usize, mut acc: u64) -> u64 {
        for e in &self.events[start..start + CHUNK] {
            // Make the input depend on the previous result, without changing
            // its distribution.
            let flip = acc & 1;
            let h = match e.kind {
                FORMAT => format::<C>(e.arg + flip as i64),
                PARSE => parse::<C>(&self.strings[e.arg as usize + flip as usize]),
                DECODE => decode::<C>(&self.columns[e.arg as usize + flip as usize..][..COLUMN]),
                SHIFT => shift::<C>(e.arg + flip as i64, e.days),
                _ => week::<C>(e.arg + flip as i64),
            };
            acc = acc.rotate_left(7) ^ h;
            acc ^= OTHERS[e.other as usize](&self.mem, acc, e.lines as u32);
        }
        acc
    }

    /// Hash of the results of the whole trace
    fn checksum<C: Calendar>(&self) -> u64 {
        (0..EVENTS).step_by(CHUNK).fold(0, |acc, start| self.run::<C>(start, acc))
    }
}

fn bench<C: Calendar + 'static>(group: &mut BenchGroup, name: &str, app: &Arc<App>) {
    let app = Arc::clone(app);
    group.bench(name.to_owned(), move |b| {
        let app = Arc::clone(&app);
        let mut start = 0;
        let mut acc = 0;
        b.iter(move || {
            acc = app.run::<C>(start, acc);
            start = (start + CHUNK) % EVENTS;
            acc
        })
    });
}

/// Register a row for each implementation, after checking that the results of
/// the trace match those of the first. A mismatch fails the benchmark for
/// datealgo implementations, and is reported for other libraries, whose
/// results may differ for reasons like leap seconds.
macro_rules! rows {
    ($group:expr, $app:expr, [$(($name:expr, $cal:ty, $exact:expr)),+ $(,)?]) => {{
        let mut baseline = None;
        $(
            let sum = $app.checksum::<$cal>();
            let expected = *baseline.get_or_insert(sum);
            if sum != expected {
                assert!(!$exact, "{}: results differ from the baseline", $name);
                eprintln!("{}: results differ from the baseline", $name);
            }
            bench::<$cal>($group, $name, &$app);
        )+
    }};
}

fn app(suite: &mut Suite) {
    let app = Arc::new(App::new());
    suite.group("app", |group| {
        group.baseline("datealgo");
        group
            .config()
            .max_rounds(50)
            .max_time(Duration::from_secs(30))
            .baseline_only(true);
        group.throughput(Throughput::Elements(CHUNK as u64)).throughput_unit("events");
        rows!(
            group,
            app,
            [
                ("datealgo", Datealgo, true),
                ("rd_table", candidates::RdTable, true),
                ("date_table", candidates::DateTable, true),
                ("leap_h64", candidates::Leap, true),
                ("copy1", candidates::Copy1, true),
                ("copy2", candidates::Copy2, true),
                ("copy3", candidates::Copy3, true),
                ("chrono", libs::Chrono, false),
                ("time", libs::Time, false),
                ("icu", libs::Icu, false),
                ("fasttime", libs::Fasttime, false),
                ("hifitime", libs::Hifitime, false),
                ("temporal", libs::Temporal, false),
            ]
        );
        bench::<NoCalendar>(group, "none", &app);
    });
}

zenbench::main!(app);
