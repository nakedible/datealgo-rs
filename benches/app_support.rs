//! The synthetic application of the application benchmarks.
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
//! The `NoCalendar` implementation does no calendar work, so the difference to
//! it is the cost of the calendar code in context.

#![allow(dead_code)]

use std::{hint::black_box, marker::PhantomData};

pub type Date = (i32, u8, u8);
pub type DateTime = (i32, u8, u8, u8, u8, u8);

/// The calendar operations of the application
pub trait Calendar {
    /// Passed to `black_box` by each event handler, so that the handlers of
    /// each implementation are distinct code that the compiler cannot merge
    /// with another's, at the same cost for all
    const SALT: u64 = 0;

    /// Unix seconds to date and time of day
    fn to_datetime(secs: i64) -> DateTime;

    /// Date and time of day to Unix seconds, if valid
    fn from_datetime(dt: DateTime) -> Option<i64>;

    /// The date and day of week some days after the date of a timestamp
    fn shift(secs: i64, days: i32) -> (Date, u8);

    /// The date of a timestamp and the six dates after it, and the day of week
    /// of the first
    fn week(secs: i64, out: &mut [Date; 7]) -> u8;
}

/// Implement `Calendar` with the public functions of a version of datealgo, as
/// a user of the library would
macro_rules! datealgo_calendar {
    ($name:ident, $salt:expr, $lib:ident) => {
        impl Calendar for $name {
            const SALT: u64 = $salt;

            fn to_datetime(secs: i64) -> DateTime {
                $lib::secs_to_datetime(secs)
            }

            fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
                if !(1..=12).contains(&m) || d < 1 || d > $lib::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
                    return None;
                }
                Some($lib::datetime_to_secs((y, m, d, hh, mm, ss)))
            }

            fn shift(secs: i64, days: i32) -> (Date, u8) {
                let n = $lib::secs_to_dhms(secs).0 + days;
                ($lib::rd_to_date(n), $lib::rd_to_weekday(n))
            }

            fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
                let n = $lib::secs_to_dhms(secs).0;
                out[0] = $lib::rd_to_date(n);
                for i in 1..7 {
                    out[i] = $lib::next_date(out[i - 1]);
                }
                $lib::rd_to_weekday(n)
            }
        }
    };
}

/// The current datealgo
pub struct Datealgo;
datealgo_calendar!(Datealgo, 0, datealgo);

/// No calendar work, as a floor for the other rows
pub struct NoCalendar;
impl Calendar for NoCalendar {
    fn to_datetime(secs: i64) -> DateTime {
        let n = secs as u64;
        (
            2026,
            (n & 7) as u8 + 1,
            (n >> 3 & 15) as u8 + 1,
            (n >> 7 & 15) as u8,
            (n >> 11 & 31) as u8,
            (n >> 16 & 31) as u8,
        )
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

// Event handlers, each a call site in the application. Each returns a hash of
// its result.

pub const DIGITS: &[u8; 200] = b"\
    0001020304050607080910111213141516171819\
    2021222324252627282930313233343536373839\
    4041424344454647484950515253545556575859\
    6061626364656667686970717273747576777879\
    8081828384858687888990919293949596979899";

pub fn put2(buf: &mut [u8], v: u8) {
    let i = v as usize % 100 * 2;
    buf[0] = DIGITS[i];
    buf[1] = DIGITS[i + 1];
}

pub fn get2(buf: &[u8]) -> Option<u8> {
    let (a, b) = (buf[0].wrapping_sub(b'0'), buf[1].wrapping_sub(b'0'));
    if a > 9 || b > 9 {
        return None;
    }
    Some(a * 10 + b)
}

pub fn hash_bytes(buf: &[u8; 20]) -> u64 {
    let a = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let b = u64::from_le_bytes(buf[8..16].try_into().unwrap());
    let c = u32::from_le_bytes(buf[16..20].try_into().unwrap()) as u64;
    a ^ b.rotate_left(21) ^ c.rotate_left(42)
}

pub fn hash_date((y, m, d): Date) -> u64 {
    (y as u64) << 16 | (m as u64) << 8 | d as u64
}

/// Format a timestamp as RFC 3339, as for a log line or a serialized value
#[inline(never)]
pub fn format<C: Calendar>(secs: i64) -> u64 {
    black_box(C::SALT);
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
pub fn parse<C: Calendar>(s: &[u8; 20]) -> u64 {
    black_box(C::SALT);
    let parsed = (|| {
        if s[4] != b'-' || s[7] != b'-' || s[10] != b'T' || s[13] != b':' || s[16] != b':' || s[19] != b'Z' {
            return None;
        }
        let y = get2(&s[0..2])? as i32 * 100 + get2(&s[2..4])? as i32;
        let dt = (
            y,
            get2(&s[5..7])?,
            get2(&s[8..10])?,
            get2(&s[11..13])?,
            get2(&s[14..16])?,
            get2(&s[17..19])?,
        );
        C::from_datetime(dt)
    })();
    parsed.map_or(0x5eed, |secs| secs as u64)
}

/// Decode a column of timestamps, as from a database
#[inline(never)]
pub fn decode<C: Calendar>(column: &[i64]) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &secs in column {
        let (y, m, d, hh, mm, ss) = C::to_datetime(secs);
        h = h.wrapping_mul(31) ^ hash_date((y, m, d)) ^ (hh as u64) << 40 ^ (mm as u64) << 48 ^ (ss as u64) << 56;
    }
    h
}

/// Find a date some days from a timestamp, as for a due date
#[inline(never)]
pub fn shift<C: Calendar>(secs: i64, days: i32) -> u64 {
    black_box(C::SALT);
    let (date, wd) = C::shift(secs, days);
    hash_date(date) ^ (wd as u64) << 32
}

/// List the dates of a week from a timestamp, as for a calendar view
#[inline(never)]
pub fn week<C: Calendar>(secs: i64) -> u64 {
    black_box(C::SALT);
    let mut dates = [(0, 0, 0); 7];
    let wd = C::week(secs, &mut dates);
    dates.iter().fold(wd as u64, |h, &d| h.rotate_left(9) ^ hash_date(d))
}

// The rest of the application

/// Words in the working set of the rest of the application, 4 MiB
pub const WORKING_SET: usize = 1 << 19;

/// Some of the rest of the application, reading `lines` cache lines from its
/// working set, with a branch on each value read. Each instance has code and
/// branches of its own.
#[inline(never)]
pub fn other<const K: u64>(mem: &[u64], seed: u64, lines: u32) -> u64 {
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

pub type Other = fn(&[u64], u64, u32) -> u64;
pub const OTHERS: [Other; 64] = others!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

// The trace

pub const FORMAT: u8 = 0;
pub const PARSE: u8 = 1;
pub const DECODE: u8 = 2;
pub const SHIFT: u8 = 3;
pub const WEEK: u8 = 4;

/// Probability of each kind of event in percent, in the order of the kinds
pub const MIX: [u32; 5] = [40, 25, 10, 15, 10];

/// Timestamps in a decoded column
pub const COLUMN: usize = 8;

/// Cache lines read by the rest of the application between two events, and
/// the probability of each in percent
pub const OTHER_LINES: [(u16, u32); 4] = [(4, 60), (16, 30), (64, 8), (1024, 2)];

/// Events in the trace, replayed in a loop
pub const EVENTS: usize = 1 << 18;

/// Events per timed iteration
pub const CHUNK: usize = 1024;

/// Size of the pools of parsed strings and decoded columns
pub const POOL: usize = 1 << 16;

#[derive(Clone, Copy)]
pub struct Event {
    kind: u8,
    other: u8,
    lines: u16,
    days: i32,
    /// A timestamp, or an index to the strings or the column pool
    arg: i64,
}

pub struct App {
    events: Vec<Event>,
    strings: Vec<[u8; 20]>,
    columns: Vec<i64>,
    mem: Vec<u64>,
}

pub fn pick<T: Copy>(choices: &[(T, u32)]) -> T {
    let mut r = fastrand::u32(0..choices.iter().map(|c| c.1).sum());
    for &(v, p) in choices {
        if r < p {
            return v;
        }
        r -= p;
    }
    unreachable!()
}

pub fn secs(dt: DateTime) -> i64 {
    datealgo::datetime_to_secs(dt)
}

impl App {
    pub fn new() -> App {
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
                    SHIFT => (
                        near_now(),
                        if fastrand::bool() {
                            fastrand::i32(1..=30)
                        } else {
                            fastrand::i32(-400..=400)
                        },
                    ),
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
    pub fn run<C: Calendar>(&self, start: usize, mut acc: u64) -> u64 {
        for (i, e) in self.events[start..start + CHUNK].iter().enumerate() {
            // Make the input wait for the previous result, as in a real
            // program, with a value that is zero unless the result happens to
            // be all ones, so that the inputs are the same for all rows.
            let dep = (acc == u64::MAX) as usize;
            let h = match e.kind {
                FORMAT => format::<C>(e.arg + dep as i64),
                PARSE => parse::<C>(&self.strings[e.arg as usize + dep]),
                DECODE => decode::<C>(&self.columns[e.arg as usize + dep..][..COLUMN]),
                SHIFT => shift::<C>(e.arg + dep as i64, e.days),
                _ => week::<C>(e.arg + dep as i64),
            };
            acc = acc.rotate_left(7) ^ h;
            // The rest of the application waits for the result in the same
            // way, and its work depends only on the event, so that it is the
            // same for all rows, even those whose results differ.
            let seed = ((start + i) as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (acc == u64::MAX) as u64;
            acc ^= OTHERS[e.other as usize](&self.mem, seed, e.lines as u32);
        }
        acc
    }

    /// Process `CHUNK` events like `run`, with one of the placed copies of the
    /// implementation in turn
    pub fn run_placed<C: Calendar>(&self, start: usize, acc: u64) -> u64 {
        match start / CHUNK % 8 {
            0 => self.run::<Placed<C, 0>>(start, acc),
            1 => self.run::<Placed<C, 1>>(start, acc),
            2 => self.run::<Placed<C, 2>>(start, acc),
            3 => self.run::<Placed<C, 3>>(start, acc),
            4 => self.run::<Placed<C, 4>>(start, acc),
            5 => self.run::<Placed<C, 5>>(start, acc),
            6 => self.run::<Placed<C, 6>>(start, acc),
            _ => self.run::<Placed<C, 7>>(start, acc),
        }
    }

    /// Hash of the results of the whole trace
    pub fn checksum<C: Calendar>(&self) -> u64 {
        (0..EVENTS).step_by(CHUNK).fold(0, |acc, start| self.run_placed::<C>(start, acc))
    }
}

/// One of several copies of an implementation. Each copy has its own copies of
/// the event handlers, placed at different addresses, and rows rotate through
/// them, so that how the code of one row happens to be placed affects its
/// results less.
pub struct Placed<C, const K: usize>(PhantomData<C>);
impl<C: Calendar, const K: usize> Calendar for Placed<C, K> {
    const SALT: u64 = C::SALT << 8 | K as u64;

    fn to_datetime(secs: i64) -> DateTime {
        C::to_datetime(secs)
    }

    fn from_datetime(dt: DateTime) -> Option<i64> {
        C::from_datetime(dt)
    }

    fn shift(secs: i64, days: i32) -> (Date, u8) {
        C::shift(secs, days)
    }

    fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
        C::week(secs, out)
    }
}
