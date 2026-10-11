//! Files, certificates and documents: ZIP entries' MS-DOS dates, directory
//! listings as ls, eza and lsd show them, tar modification times, X.509
//! validity periods, Excel serial dates as calamine and rust_xlsxwriter
//! convert them, git log dates, and PDF dates.

use super::text::{self, digits, Buf, Scan, MONTHS, MONTH_NAMES, WEEKDAYS, WEEKDAY_NAMES};
use super::*;
use std::{
    fmt::Write,
    hint::black_box,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy)]
enum Event {
    /// A ZIP central directory entry's MS-DOS date and time, to a SystemTime
    ZipRead(u16, u16),
    /// A file's modification time to an MS-DOS date and time
    ZipWrite(SystemTime),
    /// A line of a long directory listing: the modification time, with the
    /// year instead of the time for files older than six months
    Listing(SystemTime, i64),
    /// A tar header's modification time to a SystemTime and back
    Tar(i64),
    /// A certificate's validity: notBefore and notAfter as UTCTime or
    /// GeneralizedTime, checked against the present
    Certificate(u32, u32, i64),
    /// An Excel serial date to a date and time, in the 1900 or 1904 system,
    /// shown with a number format such as "dddd, mmmm d, yyyy"
    ExcelRead(f64, bool),
    /// A date and time to an Excel serial date
    ExcelWrite(DateTime),
    /// A git log line's date: "Sat Oct 11 12:34:56 2026 +0200"
    GitLog(i64, i32),
    /// A PDF date with optional fields: "D:20261011123456+02'00'"
    PdfParse(u32),
    PdfFormat(i64, i32),
}

pub struct Files {
    events: Vec<Event>,
    strings: Vec<String>,
}

impl Workload for Files {
    const NAME: &'static str = "files";
    const WEIGHT: u32 = 10;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let mut strings = Vec::new();
        let file_time = |rng: &mut fastrand::Rng| {
            NOW - {
                let c = [(rng.i64(0..30 * DAY), 40), (rng.i64(0..2 * YEAR), 40), (rng.i64(0..20 * YEAR), 20)];
                pick(rng, &c)
            }
        };
        let events = (0..EVENTS)
            .map(|_| {
                match pick(
                    rng,
                    &[(0, 15), (1, 8), (2, 15), (3, 8), (4, 12), (5, 12), (6, 6), (7, 10), (8, 7), (9, 7)],
                ) {
                    0 => {
                        let (y, m, d, hh, mm, ss) = datealgo::secs_to_datetime(file_time(rng).max(315_532_800));
                        let mut date = ((y - 1980) as u16) << 9 | (m as u16) << 5 | d as u16;
                        // Some archives have zero or bad dates
                        if rng.u32(0..50) == 0 {
                            date = rng.u16(..);
                        }
                        Event::ZipRead(date, (hh as u16) << 11 | (mm as u16) << 5 | (ss as u16 / 2))
                    }
                    1 => Event::ZipWrite(UNIX_EPOCH + Duration::from_secs(file_time(rng).max(315_532_800) as u64)),
                    2 => Event::Listing(UNIX_EPOCH + Duration::new(file_time(rng) as u64, rng.u32(0..1_000_000_000)), NOW),
                    3 => Event::Tar(file_time(rng)),
                    4 => {
                        let not_before = NOW - rng.i64(0..3 * YEAR);
                        let not_after = if rng.u32(0..20) == 0 {
                            253_402_300_799
                        } else {
                            not_before + rng.i64(90 * DAY..10 * YEAR)
                        };
                        strings.push(asn1_time(not_before));
                        strings.push(asn1_time(not_after));
                        Event::Certificate(strings.len() as u32 - 2, strings.len() as u32 - 1, NOW)
                    }
                    5 => Event::ExcelRead(
                        rng.f64() * 73_000.0 + if rng.u32(0..20) == 0 { 0.0 } else { 18_000.0 },
                        rng.u32(0..20) == 0,
                    ),
                    6 => Event::ExcelWrite(datealgo::secs_to_datetime(rng.i64(-2_208_988_800..4_102_444_800))),
                    7 => Event::GitLog(NOW - rng.i64(0..5 * YEAR), 3600 * rng.i32(-8..=10)),
                    8 => {
                        let t = NOW - rng.i64(0..10 * YEAR);
                        let (y, m, d, hh, mm, ss) = datealgo::secs_to_datetime(t);
                        let s = match rng.u32(0..4) {
                            0 => format!("D:{y:04}{m:02}{d:02}"),
                            1 => format!("D:{y:04}{m:02}{d:02}{hh:02}{mm:02}{ss:02}Z"),
                            _ => format!("D:{y:04}{m:02}{d:02}{hh:02}{mm:02}{ss:02}+0{}'00'", rng.u32(0..10)),
                        };
                        strings.push(s);
                        Event::PdfParse(strings.len() as u32 - 1)
                    }
                    _ => Event::PdfFormat(NOW - rng.i64(0..DAY), 3600 * rng.i32(-8..=10)),
                }
            })
            .collect();
        Files { events, strings }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let dep_i = dep as i64;
        let s = |k: u32| self.strings[k as usize + dep as usize].as_bytes();
        match self.events[i % EVENTS] {
            Event::ZipRead(date, time) => zip_read::<C>(date + dep as u16, time),
            Event::ZipWrite(st) => zip_write::<C>(st + Duration::from_nanos(dep)),
            Event::Listing(st, now) => listing::<C>(st + Duration::from_nanos(dep), now),
            Event::Tar(t) => tar::<C>(t + dep_i),
            Event::Certificate(a, b, now) => certificate::<C>(s(a), s(b), now),
            Event::ExcelRead(serial, s1904) => excel_read::<C>(serial + dep as f64, s1904),
            Event::ExcelWrite((y, m, d, hh, mm, ss)) => excel_write::<C>((y, m, d, hh, mm, ss + dep as u8)),
            Event::GitLog(t, offset) => git_log::<C>(t + dep_i, offset),
            Event::PdfParse(k) => pdf_parse::<C>(s(k)),
            Event::PdfFormat(t, offset) => pdf_format::<C>(t + dep_i, offset),
        }
    }
}

#[inline(never)]
fn zip_read<C: Calendar>(date: u16, time: u16) -> u64 {
    black_box(C::SALT);
    let (y, m, d) = (1980 + (date >> 9) as i32, (date >> 5 & 15) as u8, (date & 31) as u8);
    let (hh, mm, ss) = ((time >> 11) as u8, (time >> 5 & 63) as u8, (time & 31) as u8 * 2);
    // Validate as the zip crate does, with the days of the month
    if !(1..=12).contains(&m) || d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
        return 0;
    }
    C::datetime_to_systemtime((y, m, d, hh, mm, ss, 0)).map_or(1, |st| st.duration_since(UNIX_EPOCH).map_or(2, |d| d.as_secs()))
}

#[inline(never)]
fn zip_write<C: Calendar>(st: SystemTime) -> u64 {
    black_box(C::SALT);
    let Some((y, m, d, hh, mm, ss, _)) = C::systemtime_to_datetime(st) else {
        return 0;
    };
    let date = ((y - 1980).clamp(0, 127) as u16) << 9 | (m as u16) << 5 | d as u16;
    let time = (hh as u16) << 11 | (mm as u16) << 5 | (ss as u16 / 2);
    (date as u64) << 16 | time as u64
}

#[inline(never)]
fn listing<C: Calendar>(st: SystemTime, now: i64) -> u64 {
    black_box(C::SALT);
    let Some((y, m, d, hh, mm, _, _)) = C::systemtime_to_datetime(st) else {
        return 0;
    };
    let secs = st.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let mut b = Buf::<16>::new();
    b.extend(MONTHS[m as usize - 1]);
    b.push(b' ');
    if d < 10 {
        b.push(b' ');
    }
    b.put_div(d as u64, if d < 10 { 1 } else { 2 });
    b.push(b' ');
    if (now - secs).abs() < 180 * DAY {
        b.put2(hh as u32);
        b.push(b':');
        b.put2(mm as u32);
    } else {
        b.push(b' ');
        b.put4(y as u32);
    }
    text::hash(b.as_bytes())
}

#[inline(never)]
fn tar<C: Calendar>(t: i64) -> u64 {
    black_box(C::SALT);
    // The header's seconds to a SystemTime for the extracted file, and the
    // file's time back to seconds when archiving
    let st = C::secs_to_systemtime((t, 0));
    st.and_then(C::systemtime_to_secs).map_or(0, |(s, n)| s as u64 ^ n as u64)
}

/// UTCTime "YYMMDDHHMMSSZ", with years 1950-2049, or GeneralizedTime
/// "YYYYMMDDHHMMSSZ", to seconds
fn parse_asn1_time<C: Calendar>(s: &[u8]) -> Option<i64> {
    let (y, rest) = match s.len() {
        13 => {
            let yy = digits(s, 0, 2)? as i32;
            (if yy >= 50 { 1900 + yy } else { 2000 + yy }, &s[2..])
        }
        15 => (digits(s, 0, 4)? as i32, &s[4..]),
        _ => return None,
    };
    let (m, d) = (digits(rest, 0, 2)? as u8, digits(rest, 2, 2)? as u8);
    let (hh, mm, ss) = (digits(rest, 4, 2)? as u8, digits(rest, 6, 2)? as u8, digits(rest, 8, 2)? as u8);
    if rest[10] != b'Z' || !(1..=12).contains(&m) || d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
        return None;
    }
    Some(C::datetime_to_secs((y, m, d, hh, mm, ss)))
}

#[inline(never)]
fn certificate<C: Calendar>(not_before: &[u8], not_after: &[u8], now: i64) -> u64 {
    black_box(C::SALT);
    match (parse_asn1_time::<C>(not_before), parse_asn1_time::<C>(not_after)) {
        (Some(a), Some(b)) => (a <= now && now <= b) as u64 | ((b - now) as u64) << 1,
        _ => 0,
    }
}

#[inline(never)]
fn excel_read<C: Calendar>(serial: f64, s1904: bool) -> u64 {
    black_box(C::SALT);
    // Days from 1899-12-30, plus 1462 in the 1904 system. Serials before 60
    // are off by one, as Excel counts 1900-02-29.
    let mut days = serial.floor() as i32;
    if s1904 {
        days += 1462;
    } else if days < 60 {
        days += 1;
    }
    let n = C::date_to_rd((1899, 12, 30)) + days;
    let secs = ((serial - serial.floor()) * 86400.0).round() as u32;
    let (y, m, d) = C::rd_to_date(n);
    let wd = C::rd_to_weekday(n);
    let mut b = Buf::<48>::new();
    let _ = write!(
        b,
        "{}, {} {d}, {y} {:02}:{:02}",
        WEEKDAY_NAMES[wd as usize - 1],
        MONTH_NAMES[m as usize - 1],
        secs / 3600,
        secs / 60 % 60
    );
    text::hash(b.as_bytes())
}

#[inline(never)]
fn excel_write<C: Calendar>((y, m, d, hh, mm, ss): DateTime) -> u64 {
    black_box(C::SALT);
    let mut days = C::date_to_rd((y, m, d)) - C::date_to_rd((1899, 12, 30));
    if days < 61 {
        days -= 1;
    }
    let serial = days as f64 + (hh as u32 * 3600 + mm as u32 * 60 + ss as u32) as f64 / 86400.0;
    serial.to_bits()
}

#[inline(never)]
fn git_log<C: Calendar>(t: i64, offset: i32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(t + offset as i64);
    let wd = C::date_to_weekday((y, m, d));
    let mut b = Buf::<40>::new();
    b.extend(WEEKDAYS[wd as usize - 1]);
    b.push(b' ');
    b.extend(MONTHS[m as usize - 1]);
    b.push(b' ');
    b.put_div(d as u64, if d < 10 { 1 } else { 2 });
    b.push(b' ');
    b.put2(hh as u32);
    b.push(b':');
    b.put2(mm as u32);
    b.push(b':');
    b.put2(ss as u32);
    b.push(b' ');
    b.put4(y as u32);
    b.push(b' ');
    b.push(if offset < 0 { b'-' } else { b'+' });
    let o = offset.unsigned_abs();
    b.put2(o / 3600);
    b.put2(o / 60 % 60);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn pdf_parse<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    // Fields after the year are optional and default to the start of the
    // period, as pdf_oxide and lopdf parse them
    let parse = || {
        let mut p = Scan::new(s);
        if !(p.eat(b'D') && p.eat(b':')) {
            return None;
        }
        let y = p.num(4, 4)? as i32;
        let m = p.num(2, 2).unwrap_or(1) as u8;
        let d = p.num(2, 2).unwrap_or(1) as u8;
        let hh = p.num(2, 2).unwrap_or(0) as u8;
        let mm = p.num(2, 2).unwrap_or(0) as u8;
        let ss = p.num(2, 2).unwrap_or(0) as u8;
        let offset = if p.done() || p.eat(b'Z') {
            0
        } else {
            let sign = if p.eat(b'-') {
                -1
            } else {
                p.eat(b'+');
                1
            };
            let oh = p.num(2, 2)? as i32;
            p.eat(b'\'');
            sign * (oh * 3600 + p.num(2, 2).unwrap_or(0) as i32 * 60)
        };
        if !(1..=12).contains(&m) || d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
            return None;
        }
        Some(C::datetime_to_secs((y, m, d, hh, mm, ss)) - offset as i64)
    };
    parse().map_or(1, |t| t as u64)
}

#[inline(never)]
fn pdf_format<C: Calendar>(t: i64, offset: i32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(t + offset as i64);
    let mut b = Buf::<24>::new();
    b.extend(b"D:");
    b.put4(y as u32);
    b.put2(m as u32);
    b.put2(d as u32);
    b.put2(hh as u32);
    b.put2(mm as u32);
    b.put2(ss as u32);
    b.push(if offset < 0 { b'-' } else { b'+' });
    let o = offset.unsigned_abs();
    b.put2(o / 3600);
    b.push(b'\'');
    b.put2(o / 60 % 60);
    b.push(b'\'');
    text::hash(b.as_bytes())
}

fn asn1_time(t: i64) -> String {
    let (y, m, d, hh, mm, ss) = datealgo::secs_to_datetime(t);
    if (1950..2050).contains(&y) {
        format!("{:02}{m:02}{d:02}{hh:02}{mm:02}{ss:02}Z", y % 100)
    } else {
        format!("{y:04}{m:02}{d:02}{hh:02}{mm:02}{ss:02}Z")
    }
}
