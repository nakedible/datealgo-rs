//! Databases and columnar engines: database drivers encoding and decoding
//! values in their wire formats (Postgres, MySQL, MSSQL, Cassandra, SQLite),
//! and column kernels as Arrow, DataFusion and Polars run them over batches:
//! extracting fields, truncating, adding months, and parsing and formatting
//! date columns.

use super::text::{self, digit_mask, digits, Buf};
use super::*;
use std::{fmt::Write, hint::black_box, time::UNIX_EPOCH};

/// Values in a column batch
const BATCH: usize = 16;

#[derive(Clone, Copy)]
enum Event {
    /// A Postgres row: a DATE as days since 2000-01-01 and a TIMESTAMP as
    /// microseconds since 2000-01-01, decoded and encoded again
    Postgres(i32, i64),
    /// A MySQL binary row: date and time fields, validated and converted, and
    /// a timestamp back to fields
    MySql(DateTime, u32, i64),
    /// An MSSQL row: a DATE as days since 0001-01-01 and a DATETIME as days
    /// since 1900-01-01 and 1/300 second ticks
    MsSql(i32, i32, u32),
    /// A Cassandra row: a date as days with 1970-01-01 at 2^31, and a
    /// timestamp in milliseconds, into a SystemTime
    Cassandra(u32, i64),
    /// SQLite text, tried against several formats as sqlx and diesel do
    Sqlite(u32),
    /// Date32 column batch at `start`: extract a field
    DatePart(u32, Part),
    /// Timestamp (ns) column batch at `start`: truncate to a unit
    DateTrunc(u32, Unit),
    /// Date32 column batch plus an interval of months, clamping to the end of
    /// the month
    AddMonths(u32, i32),
    /// CSV date column batch of strings at `start`, parsed to Date32
    CsvParse(u32),
    /// Date32 column batch at `start` formatted as ISO dates
    CsvFormat(u32),
    /// Polars is_leap_year and days_in_month over a Date32 batch
    MonthInfo(u32),
}

#[derive(Clone, Copy)]
enum Part {
    Year,
    Month,
    Day,
    DayOfWeek,
    DayOfYear,
    Quarter,
    IsoWeek,
    IsoYear,
}

#[derive(Clone, Copy)]
enum Unit {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

pub struct Data {
    events: Vec<Event>,
    /// Date32 values
    dates: Vec<i32>,
    /// Timestamp values in nanoseconds
    stamps: Vec<i64>,
    /// Date strings for CSV and SQLite
    strings: Vec<String>,
}

impl Workload for Data {
    const NAME: &'static str = "data";
    const WEIGHT: u32 = 25;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let (mut dates, mut stamps, mut strings) = (Vec::new(), Vec::new(), Vec::new());
        let rd = |d| datealgo::date_to_rd(d);
        let mut events = Vec::with_capacity(EVENTS);
        for _ in 0..EVENTS {
            let kind = pick(
                rng,
                &[
                    (0, 18),
                    (1, 12),
                    (2, 6),
                    (3, 6),
                    (4, 6),
                    (5, 14),
                    (6, 10),
                    (7, 6),
                    (8, 10),
                    (9, 7),
                    (10, 5),
                ],
            );
            let ev = match kind {
                0 => {
                    let day = rd(data_date(rng)) - 10_957;
                    let us = (near_now(rng, YEAR) - 946_684_800) * 1_000_000 + rng.i64(0..1_000_000);
                    Event::Postgres(day, us)
                }
                1 => {
                    let (y, m, d) = data_date(rng);
                    let mut dt = (y, m, d, rng.u8(0..24), rng.u8(0..60), rng.u8(0..60));
                    // Invalid dates come from zero dates and bad input
                    if rng.u32(0..100) == 0 {
                        dt.2 = 31;
                    }
                    Event::MySql(dt, rng.u32(0..1_000_000), near_now(rng, YEAR))
                }
                2 => Event::MsSql(rd(data_date(rng)) + 719_162, rd(data_date(rng)) + 25_567, rng.u32(0..25_920_000)),
                3 => Event::Cassandra(
                    (rd(data_date(rng)) as i64 + (1 << 31)) as u32,
                    near_now(rng, YEAR) * 1000 + rng.i64(0..1000),
                ),
                4 => {
                    let t = near_now(rng, 2 * YEAR);
                    let (y, m, d, hh, mm, ss) = datealgo::secs_to_datetime(t);
                    let s = match rng.u32(0..4) {
                        0 => format!("{y:04}-{m:02}-{d:02}"),
                        1 => format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}"),
                        2 => format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}.{:06}", rng.u32(0..1_000_000)),
                        _ => format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{:03}Z", rng.u32(0..1000)),
                    };
                    strings.push(s);
                    Event::Sqlite(strings.len() as u32 - 1)
                }
                5 | 7 | 9 | 10 => {
                    let start = dates.len() as u32;
                    for _ in 0..BATCH {
                        dates.push(rd(data_date(rng)));
                    }
                    match kind {
                        5 => Event::DatePart(
                            start,
                            pick(
                                rng,
                                &[
                                    (Part::Year, 25),
                                    (Part::Month, 20),
                                    (Part::Day, 10),
                                    (Part::DayOfWeek, 15),
                                    (Part::DayOfYear, 5),
                                    (Part::Quarter, 10),
                                    (Part::IsoWeek, 10),
                                    (Part::IsoYear, 5),
                                ],
                            ),
                        ),
                        7 => Event::AddMonths(start, {
                            let c = [(1, 40), (3, 20), (12, 20), (-1, 10), (rng.i32(-24..=24), 10)];
                            pick(rng, &c)
                        }),
                        9 => Event::CsvFormat(start),
                        _ => Event::MonthInfo(start),
                    }
                }
                6 => {
                    let start = stamps.len() as u32;
                    for _ in 0..BATCH {
                        let t = if rng.u32(0..20) == 0 {
                            rng.i64(-30 * YEAR..0)
                        } else {
                            near_now(rng, 2 * YEAR)
                        };
                        stamps.push(t * 1_000_000_000 + rng.i64(0..1_000_000_000));
                    }
                    Event::DateTrunc(
                        start,
                        pick(
                            rng,
                            &[
                                (Unit::Day, 30),
                                (Unit::Week, 15),
                                (Unit::Month, 30),
                                (Unit::Quarter, 10),
                                (Unit::Year, 15),
                            ],
                        ),
                    )
                }
                _ => {
                    let start = strings.len() as u32;
                    for _ in 0..BATCH {
                        let (y, m, d) = data_date(rng);
                        strings.push(if rng.u32(0..10) == 0 {
                            format!("{y:04}{m:02}{d:02}")
                        } else {
                            format!("{y:04}-{m:02}-{d:02}")
                        });
                    }
                    Event::CsvParse(start)
                }
            };
            events.push(ev);
        }
        Data {
            events,
            dates,
            stamps,
            strings,
        }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let (dep_i, dep_u) = (dep as i64, dep as usize);
        let batch = |s: u32| s as usize + dep_u..s as usize + BATCH;
        match self.events[i % EVENTS] {
            Event::Postgres(day, us) => postgres::<C>(day + dep as i32, us),
            Event::MySql(dt, us, ts) => mysql::<C>(dt, us, ts + dep_i),
            Event::MsSql(date, days, ticks) => mssql::<C>(date + dep as i32, days, ticks),
            Event::Cassandra(date, ms) => cassandra::<C>(date + dep as u32, ms),
            Event::Sqlite(k) => sqlite::<C>(&self.strings[k as usize + dep_u]),
            Event::DatePart(s, part) => date_part::<C>(&self.dates[batch(s)], part),
            Event::DateTrunc(s, unit) => date_trunc::<C>(&self.stamps[batch(s)], unit),
            Event::AddMonths(s, months) => add_months::<C>(&self.dates[batch(s)], months),
            Event::CsvParse(s) => csv_parse::<C>(&self.strings[batch(s)]),
            Event::CsvFormat(s) => csv_format::<C>(&self.dates[batch(s)]),
            Event::MonthInfo(s) => month_info::<C>(&self.dates[batch(s)]),
        }
    }
}

#[inline(never)]
fn postgres<C: Calendar>(day: i32, us: i64) -> u64 {
    black_box(C::SALT);
    // Decode the date from its epoch, which drivers build from the date each
    // time, and the timestamp by splitting off the microseconds
    let epoch = C::date_to_rd((2000, 1, 1));
    let date = C::rd_to_date(epoch + day);
    let secs = us.div_euclid(1_000_000) + C::datetime_to_secs((2000, 1, 1, 0, 0, 0));
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs);
    // Encode both again
    let day2 = C::date_to_rd(date) - C::date_to_rd((2000, 1, 1));
    let us2 =
        (C::datetime_to_secs((y, m, d, hh, mm, ss)) - C::datetime_to_secs((2000, 1, 1, 0, 0, 0))) * 1_000_000 + us.rem_euclid(1_000_000);
    text::mix(day2 as u64, us2 as u64)
}

#[inline(never)]
fn mysql<C: Calendar>((y, m, d, hh, mm, ss): DateTime, us: u32, ts: i64) -> u64 {
    black_box(C::SALT);
    // Decode: validate the fields and convert, as from_ymd_opt and and_hms do
    let decoded = if (1..=12).contains(&m) && d >= 1 && d <= C::days_in_month(y, m) {
        C::datetime_to_systemtime((y, m, d, hh, mm, ss, us * 1000))
            .map_or(1, |st| st.duration_since(UNIX_EPOCH).map_or(2, |x| x.as_micros() as u64))
    } else {
        0
    };
    // Encode a timestamp into the fields of the binary protocol
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(ts);
    let mut row = [0u8; 8];
    row[..2].copy_from_slice(&(y as u16).to_le_bytes());
    row[2..].copy_from_slice(&[m, d, hh, mm, ss, 0]);
    text::mix(decoded, u64::from_le_bytes(row))
}

#[inline(never)]
fn mssql<C: Calendar>(date: i32, days: i32, ticks: u32) -> u64 {
    black_box(C::SALT);
    // DATE from 0001-01-01 and DATETIME from 1900-01-01, as tiberius builds
    // each epoch and adds the days
    let d = C::rd_to_date(C::date_to_rd((1, 1, 1)) + date);
    let base = C::date_to_rd((1900, 1, 1));
    let secs = C::dhms_to_secs((base + days, 0, 0, 0)) + (ticks / 300) as i64;
    let (y, m, dd, hh, mm, ss) = C::secs_to_datetime(secs);
    text::mix(
        text::mix(d.0 as u64, (d.1 as u64) << 8 | d.2 as u64),
        dt_bits((y, m, dd, hh, mm, ss)),
    )
}

#[inline(never)]
fn cassandra<C: Calendar>(date: u32, ms: i64) -> u64 {
    black_box(C::SALT);
    let d = C::rd_to_date((date as i64 - (1 << 31)) as i32);
    let st = C::secs_to_systemtime((ms.div_euclid(1000), ms.rem_euclid(1000) as u32 * 1_000_000));
    let dt = st.and_then(C::systemtime_to_datetime);
    text::mix(
        (d.0 as u64) << 16 | (d.1 as u64) << 8 | d.2 as u64,
        dt.map_or(1, |(y, m, d, hh, mm, ss, _)| dt_bits((y, m, d, hh, mm, ss))),
    )
}

#[inline(never)]
fn sqlite<C: Calendar>(s: &str) -> u64 {
    black_box(C::SALT);
    // Try the formats in turn by splitting the text into numbers, as
    // application code does
    let s = s.trim_end_matches('Z');
    let r = if let Some([y, m, d, hh, mm, ss, _]) = text::split_numbers::<7>(s, &['-', ' ', 'T', ':', '.']) {
        Some((y, m, d, hh, mm, ss))
    } else if let Some([y, m, d, hh, mm, ss]) = text::split_numbers::<6>(s, &['-', ' ', 'T', ':']) {
        Some((y, m, d, hh, mm, ss))
    } else if let Some([y, m, d]) = text::split_numbers::<3>(s, &['-']) {
        Some((y, m, d, 0, 0, 0))
    } else {
        None
    };
    match r {
        Some((y, m, d, hh, mm, ss)) if (1..=12).contains(&m) && d >= 1 && d <= C::days_in_month(y as i32, m as u8) as u32 => {
            C::datetime_to_secs((y as i32, m as u8, d as u8, hh as u8, mm as u8, ss as u8)) as u64
        }
        _ => 1,
    }
}

#[inline(never)]
fn date_part<C: Calendar>(dates: &[i32], part: Part) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &n in dates {
        let v = match part {
            Part::Year => C::rd_to_date(n).0,
            Part::Month => C::rd_to_date(n).1 as i32,
            Part::Day => C::rd_to_date(n).2 as i32,
            Part::DayOfWeek => C::rd_to_weekday(n) as i32 % 7,
            Part::DayOfYear => {
                let (y, _, _) = C::rd_to_date(n);
                n - C::date_to_rd((y, 1, 1)) + 1
            }
            Part::Quarter => (C::rd_to_date(n).1 as i32 - 1) / 3 + 1,
            Part::IsoWeek => C::rd_to_isoweekdate(n).1 as i32,
            Part::IsoYear => C::rd_to_isoweekdate(n).0,
        };
        h = h.wrapping_mul(31).wrapping_add(v as u64);
    }
    h
}

#[inline(never)]
fn date_trunc<C: Calendar>(stamps: &[i64], unit: Unit) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &ns in stamps {
        let (days, _, _, _) = C::secs_to_dhms(ns.div_euclid(1_000_000_000));
        let start = match unit {
            Unit::Day => days,
            Unit::Week => days - (C::rd_to_weekday(days) as i32 - 1),
            Unit::Month => {
                let (_, _, d) = C::rd_to_date(days);
                days - (d as i32 - 1)
            }
            Unit::Quarter => {
                let (y, m, _) = C::rd_to_date(days);
                C::date_to_rd((y, (m - 1) / 3 * 3 + 1, 1))
            }
            Unit::Year => C::date_to_rd((C::rd_to_date(days).0, 1, 1)),
        };
        h = h.wrapping_mul(31).wrapping_add(C::dhms_to_secs((start, 0, 0, 0)) as u64);
    }
    h
}

#[inline(never)]
fn add_months<C: Calendar>(dates: &[i32], months: i32) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &n in dates {
        let (y, m, d) = C::rd_to_date(n);
        let total = y * 12 + m as i32 - 1 + months;
        let (y, m) = (total.div_euclid(12), total.rem_euclid(12) as u8 + 1);
        let d = d.min(C::days_in_month(y, m));
        h = h.wrapping_mul(31).wrapping_add(C::date_to_rd((y, m, d)) as u64);
    }
    h
}

#[inline(never)]
fn csv_parse<C: Calendar>(strings: &[String]) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for s in strings {
        let s = s.as_bytes();
        // Check all digit positions at once, as arrow-cast does
        let ymd = if s.len() == 10 && s[4] == b'-' && s[7] == b'-' && digit_mask(s, 0b11_0110_1111) {
            Some((
                digits(s, 0, 4).unwrap_or(0) as i32,
                digits(s, 5, 2).unwrap_or(0) as u8,
                digits(s, 8, 2).unwrap_or(0) as u8,
            ))
        } else if s.len() == 8 && digit_mask(s, 0xff) {
            Some((
                digits(s, 0, 4).unwrap_or(0) as i32,
                digits(s, 4, 2).unwrap_or(0) as u8,
                digits(s, 6, 2).unwrap_or(0) as u8,
            ))
        } else {
            None
        };
        let v = match ymd {
            Some((y, m, d)) if (1..=12).contains(&m) && d >= 1 && d <= C::days_in_month(y, m) => C::date_to_rd((y, m, d)),
            _ => i32::MIN,
        };
        h = h.wrapping_mul(31).wrapping_add(v as u64);
    }
    h
}

#[inline(never)]
fn csv_format<C: Calendar>(dates: &[i32]) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &n in dates {
        let (y, m, d) = C::rd_to_date(n);
        let mut b = Buf::<16>::new();
        let _ = write!(b, "{y:04}-{m:02}-{d:02}");
        h = text::mix(h, text::hash(b.as_bytes()));
    }
    h
}

#[inline(never)]
fn month_info<C: Calendar>(dates: &[i32]) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    for &n in dates {
        let (y, m, _) = C::rd_to_date(n);
        h = h
            .wrapping_mul(31)
            .wrapping_add(C::is_leap_year(y) as u64 * 64 + C::days_in_month(y, m) as u64);
    }
    h
}

fn dt_bits((y, m, d, hh, mm, ss): DateTime) -> u64 {
    (y as u64) << 40 | (m as u64) << 32 | (d as u64) << 24 | (hh as u64) << 16 | (mm as u64) << 8 | ss as u64
}
