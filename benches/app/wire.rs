//! Web, API and messaging formats: Kubernetes objects with many RFC 3339
//! timestamps, protobuf timestamps to and from JSON as prost-types does it,
//! HTTP dates in headers and cookies as httpdate, cookie and the time crate do
//! it, AWS signatures, email Date headers and IMAP dates, and JWT claims.

use super::text::{self, digits, Buf, Scan, MONTHS, WEEKDAYS};
use super::*;
use std::{
    hint::black_box,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy)]
enum Event {
    /// A Kubernetes object with timestamps "2026-10-11T12:34:56Z" and
    /// "2026-10-11T12:34:56.123456Z" at strings `start..start+count`
    K8sObject(u32, u8),
    /// A protobuf Timestamp to JSON, with 0, 3, 6 or 9 fraction digits as
    /// needed
    ProtoToJson(i64, u32),
    /// A JSON RFC 3339 string to a protobuf Timestamp, with offsets
    JsonToProto(u32),
    /// An HTTP date of a file's modification time or a cookie's expiry, the
    /// way httpdate formats it
    HttpDateFormat(SystemTime),
    /// An HTTP date the way the time crate formats it
    HttpDateFormatTime(i64),
    /// An HTTP date from a request or cookie: IMF-fixdate, RFC 850 or asctime
    HttpDateParse(u32),
    /// An AWS signature's date and timestamp, converted separately
    SigV4(SystemTime),
    /// An email Date header: "Sat, 11 Oct 2026 14:34:56 +0200"
    EmailFormat(i64, i32),
    EmailParse(u32),
    /// An IMAP INTERNALDATE: "11-Oct-2026 14:34:56 +0200"
    ImapParse(u32),
    /// JWT claims exp, iat and nbf converted to date-times and checked
    Jwt(i64, i64, i64, i64),
}

pub struct Wire {
    events: Vec<Event>,
    strings: Vec<String>,
}

impl Workload for Wire {
    const NAME: &'static str = "wire";
    const WEIGHT: u32 = 20;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let mut strings = Vec::new();
        let mut events = Vec::with_capacity(EVENTS);
        for _ in 0..EVENTS {
            let ev = match pick(
                rng,
                &[
                    (0, 20),
                    (1, 12),
                    (2, 10),
                    (3, 10),
                    (4, 5),
                    (5, 10),
                    (6, 10),
                    (7, 5),
                    (8, 6),
                    (9, 4),
                    (10, 8),
                ],
            ) {
                0 => {
                    let start = strings.len() as u32;
                    let count = rng.u8(5..=20);
                    let created = NOW - rng.i64(0..2 * YEAR);
                    for _ in 0..count {
                        let t = created + rng.i64(0..(NOW - created + 1));
                        let micro = rng.u32(0..4) == 0;
                        strings.push(rfc3339(
                            t,
                            if micro { rng.u32(0..1_000_000) * 1000 } else { 0 },
                            if micro { 6 } else { 0 },
                            0,
                        ));
                    }
                    Event::K8sObject(start, count)
                }
                1 => Event::ProtoToJson(value_time(rng), {
                    let c = [(0, 40), (rng.u32(0..1000) * 1_000_000, 30), (rng.u32(0..1_000_000_000), 30)];
                    pick(rng, &c)
                }),
                2 => {
                    let t = value_time(rng);
                    let digits = pick(rng, &[(0, 40), (3, 30), (6, 20), (9, 10)]);
                    let offset = {
                        let c = [(0, 60), (3600 * rng.i32(-11..=12), 40)];
                        pick(rng, &c)
                    };
                    let mut s = rfc3339(t, rng.u32(0..1_000_000_000), digits, offset);
                    if rng.u32(0..20) == 0 {
                        s = s.to_lowercase();
                    }
                    strings.push(invalidate(rng, s));
                    Event::JsonToProto(strings.len() as u32 - 1)
                }
                3 => {
                    let t = match rng.u32(0..3) {
                        0 => NOW - rng.i64(0..5 * YEAR),
                        1 => NOW + rng.i64(DAY..YEAR),
                        _ => NOW + rng.i64(0..DAY),
                    };
                    Event::HttpDateFormat(UNIX_EPOCH + Duration::from_secs(t as u64))
                }
                4 => Event::HttpDateFormatTime(NOW + rng.i64(-YEAR..YEAR)),
                5 => {
                    let t = match rng.u32(0..3) {
                        0 => NOW - rng.i64(0..5 * YEAR),
                        _ => NOW + rng.i64(0..YEAR),
                    };
                    let s = match pick(rng, &[(0, 85), (1, 10), (2, 5)]) {
                        0 => http_date(t),
                        1 => rfc850(t),
                        _ => asctime(t),
                    };
                    strings.push(invalidate(rng, s));
                    Event::HttpDateParse(strings.len() as u32 - 1)
                }
                6 => Event::SigV4(
                    UNIX_EPOCH + Duration::from_nanos((NOW + rng.i64(-DAY..DAY)) as u64 * 1_000_000_000 + rng.u64(0..1_000_000_000)),
                ),
                7 => Event::EmailFormat(NOW + rng.i64(-DAY..DAY), 3600 * rng.i32(-8..=9)),
                8 => {
                    let t = NOW - rng.i64(0..3 * YEAR);
                    let s = email_date(t, 3600 * rng.i32(-8..=9), rng.bool());
                    strings.push(invalidate(rng, s));
                    Event::EmailParse(strings.len() as u32 - 1)
                }
                9 => {
                    let t = NOW - rng.i64(0..3 * YEAR);
                    strings.push(imap_date(t, 3600 * rng.i32(-8..=9)));
                    Event::ImapParse(strings.len() as u32 - 1)
                }
                _ => {
                    let iat = NOW - rng.i64(0..3600);
                    Event::Jwt(iat + 3600, iat, iat - 60, NOW)
                }
            };
            events.push(ev);
        }
        Wire { events, strings }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let dep_i = dep as i64;
        let s = |k: u32| self.strings[k as usize + dep as usize].as_bytes();
        match self.events[i % EVENTS] {
            Event::K8sObject(start, count) => {
                k8s_object::<C>(&self.strings[start as usize + dep as usize..(start as usize + count as usize)])
            }
            Event::ProtoToJson(secs, nanos) => proto_to_json::<C>(secs + dep_i, nanos),
            Event::JsonToProto(k) => json_to_proto::<C>(s(k)),
            Event::HttpDateFormat(st) => http_date_format::<C>(st + Duration::from_nanos(dep)),
            Event::HttpDateFormatTime(secs) => http_date_format_time::<C>(secs + dep_i),
            Event::HttpDateParse(k) => http_date_parse::<C>(s(k)),
            Event::SigV4(st) => sigv4::<C>(st + Duration::from_nanos(dep)),
            Event::EmailFormat(secs, offset) => email_format::<C>(secs + dep_i, offset),
            Event::EmailParse(k) => email_parse::<C>(s(k)),
            Event::ImapParse(k) => imap_parse::<C>(s(k)),
            Event::Jwt(exp, iat, nbf, now) => jwt::<C>(exp + dep_i, iat, nbf, now),
        }
    }
}

// Handlers

#[inline(never)]
fn k8s_object<C: Calendar>(fields: &[String]) -> u64 {
    black_box(C::SALT);
    let mut h = 0;
    for f in fields {
        h = text::mix(h, parse_rfc3339_strict::<C>(f.as_bytes()).map_or(1, |(s, n)| s as u64 ^ n as u64));
    }
    h
}

/// "YYYY-MM-DDTHH:MM:SS[.f]Z" at fixed positions, as jiff and chrono parse
/// Kubernetes timestamps
fn parse_rfc3339_strict<C: Calendar>(s: &[u8]) -> Option<(i64, u32)> {
    if s.len() < 20 || s[4] != b'-' || s[7] != b'-' || s[10] != b'T' || s[13] != b':' || s[16] != b':' {
        return None;
    }
    let (y, m, d) = (digits(s, 0, 4)? as i32, digits(s, 5, 2)? as u8, digits(s, 8, 2)? as u8);
    let (hh, mm, ss) = (digits(s, 11, 2)? as u8, digits(s, 14, 2)? as u8, digits(s, 17, 2)? as u8);
    let mut scan = Scan { s, i: 19 };
    let nanos = if scan.eat(b'.') { scan.frac() } else { 0 };
    if !scan.eat(b'Z') || !scan.done() || !(1..=12).contains(&m) || d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
        return None;
    }
    Some((C::datetime_to_secs((y, m, d, hh, mm, ss)), nanos))
}

#[inline(never)]
fn proto_to_json<C: Calendar>(secs: i64, nanos: u32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs);
    let mut b = Buf::<40>::new();
    b.put_year(y);
    b.push(b'-');
    b.put2(m as u32);
    b.push(b'-');
    b.put2(d as u32);
    b.push(b'T');
    b.put2(hh as u32);
    b.push(b':');
    b.put2(mm as u32);
    b.push(b':');
    b.put2(ss as u32);
    if nanos != 0 {
        let digits = if nanos % 1_000_000 == 0 {
            3
        } else if nanos % 1000 == 0 {
            6
        } else {
            9
        };
        b.put_frac(nanos, digits);
    }
    b.push(b'Z');
    text::hash(b.as_bytes())
}

#[inline(never)]
fn json_to_proto<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    parse_rfc3339::<C>(s).map_or(1, |(secs, nanos)| secs as u64 ^ (nanos as u64) << 32)
}

/// RFC 3339 with any fraction and offset, lowercase `t` and `z`, a space as
/// the separator, and a leap second clamped to 59, as prost-types parses it
fn parse_rfc3339<C: Calendar>(s: &[u8]) -> Option<(i64, u32)> {
    let mut p = Scan::new(s);
    let y = p.num(4, 4)? as i32;
    let m = p.eat(b'-').then(|| p.num(2, 2))?? as u8;
    let d = p.eat(b'-').then(|| p.num(2, 2))?? as u8;
    if !p.eat_any(b"Tt ") {
        return None;
    }
    let hh = p.num(2, 2)? as u8;
    let mm = p.eat(b':').then(|| p.num(2, 2))?? as u8;
    let ss = p.eat(b':').then(|| p.num(2, 2))?? as u8;
    let nanos = if p.eat(b'.') { p.frac() } else { 0 };
    let offset = p.offset()?;
    if !p.done() || !(1..=12).contains(&m) || d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 60 {
        return None;
    }
    Some((C::datetime_to_secs((y, m, d, hh, mm, ss.min(59))) - offset as i64, nanos))
}

#[inline(never)]
fn http_date_format<C: Calendar>(st: SystemTime) -> u64 {
    black_box(C::SALT);
    // As httpdate does: days and seconds of the day from the seconds, then the
    // date and the day of the week from the days
    let Some((secs, _)) = C::systemtime_to_secs(st) else {
        return 0;
    };
    let (days, hh, mm, ss) = C::secs_to_dhms(secs);
    let (y, m, d) = C::rd_to_date(days);
    let wd = C::rd_to_weekday(days);
    let mut b = Buf::<32>::new();
    b.extend(WEEKDAYS[wd as usize - 1]);
    b.extend(b", ");
    b.put2(d as u32);
    b.push(b' ');
    b.extend(MONTHS[m as usize - 1]);
    b.push(b' ');
    b.put4(y as u32);
    b.push(b' ');
    b.put2(hh as u32);
    b.push(b':');
    b.put2(mm as u32);
    b.push(b':');
    b.put2(ss as u32);
    b.extend(b" GMT");
    text::hash(b.as_bytes())
}

#[inline(never)]
fn http_date_format_time<C: Calendar>(secs: i64) -> u64 {
    black_box(C::SALT);
    // As the time crate does: the date and time, and the day of the week from
    // the date
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs);
    let wd = C::date_to_weekday((y, m, d));
    let mut b = Buf::<32>::new();
    use std::fmt::Write;
    let _ = write!(
        b,
        "{}, {d:02} {} {y} {hh:02}:{mm:02}:{ss:02} GMT",
        std::str::from_utf8(WEEKDAYS[wd as usize - 1]).unwrap_or(""),
        std::str::from_utf8(MONTHS[m as usize - 1]).unwrap_or("")
    );
    text::hash(b.as_bytes())
}

#[inline(never)]
fn http_date_parse<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    parse_http_date::<C>(s)
        .and_then(|dt| C::datetime_to_systemtime(dt))
        .map_or(1, |st| st.duration_since(UNIX_EPOCH).map_or(2, |d| d.as_secs()))
}

/// IMF-fixdate, RFC 850 and asctime, checking the day of the week as httpdate
/// does, with two-digit years as cookies have them
fn parse_http_date<C: Calendar>(s: &[u8]) -> Option<DateTimeNanos> {
    let mut p = Scan::new(s);
    let wd = p.weekday_name()?;
    let (y, m, d);
    if p.eat(b',') {
        p.skip_spaces();
        d = p.num(1, 2)? as u8;
        if p.eat(b'-') {
            // RFC 850: "Sunday, 06-Nov-94 08:49:37 GMT"
            m = p.month_name()?;
            p.eat(b'-');
            let yy = p.num(2, 4)? as i32;
            y = if yy < 70 {
                2000 + yy
            } else if yy < 100 {
                1900 + yy
            } else {
                yy
            };
        } else {
            p.skip_spaces();
            m = p.month_name()?;
            p.skip_spaces();
            y = p.num(4, 4)? as i32;
        }
    } else {
        // asctime: "Sun Nov  6 08:49:37 1994"
        p.skip_spaces();
        m = p.month_name()?;
        p.skip_spaces();
        d = p.num(1, 2)? as u8;
        let mut q = Scan {
            s,
            i: s.len().checked_sub(4)?,
        };
        y = q.num(4, 4)? as i32;
    }
    p.skip_spaces();
    let hh = p.num(2, 2)? as u8;
    let mm = p.eat(b':').then(|| p.num(2, 2))?? as u8;
    let ss = p.eat(b':').then(|| p.num(2, 2))?? as u8;
    if d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 || C::date_to_weekday((y, m, d)) != wd {
        return None;
    }
    Some((y, m, d, hh, mm, ss, 0))
}

#[inline(never)]
fn sigv4<C: Calendar>(st: SystemTime) -> u64 {
    black_box(C::SALT);
    // aws-sigv4 converts the time separately for the date and the timestamp
    let Some((y, m, d, _, _, _, _)) = C::systemtime_to_datetime(st) else {
        return 0;
    };
    let mut date = Buf::<8>::new();
    date.put4(y as u32);
    date.put2(m as u32);
    date.put2(d as u32);
    let Some((y, m, d, hh, mm, ss, _)) = C::systemtime_to_datetime(st) else {
        return 0;
    };
    let mut stamp = Buf::<16>::new();
    stamp.put4(y as u32);
    stamp.put2(m as u32);
    stamp.put2(d as u32);
    stamp.push(b'T');
    stamp.put2(hh as u32);
    stamp.put2(mm as u32);
    stamp.put2(ss as u32);
    stamp.push(b'Z');
    text::mix(text::hash(date.as_bytes()), text::hash(stamp.as_bytes()))
}

#[inline(never)]
fn email_format<C: Calendar>(secs: i64, offset: i32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs + offset as i64);
    let wd = C::date_to_weekday((y, m, d));
    let mut b = Buf::<40>::new();
    b.extend(WEEKDAYS[wd as usize - 1]);
    b.extend(b", ");
    b.put_div(d as u64, if d < 10 { 1 } else { 2 });
    b.push(b' ');
    b.extend(MONTHS[m as usize - 1]);
    b.push(b' ');
    b.put4(y as u32);
    b.push(b' ');
    b.put2(hh as u32);
    b.push(b':');
    b.put2(mm as u32);
    b.push(b':');
    b.put2(ss as u32);
    b.push(b' ');
    b.push(if offset < 0 { b'-' } else { b'+' });
    let o = offset.unsigned_abs();
    b.put2(o / 3600);
    b.put2(o / 60 % 60);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn email_parse<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    // RFC 2822, with an optional day of the week that is not checked, as the
    // email crate does, and two-digit years
    let parse = || {
        let mut p = Scan::new(s);
        if p.weekday_name().is_some() {
            p.eat(b',');
            p.skip_spaces();
        }
        let d = p.num(1, 2)? as u8;
        p.skip_spaces();
        let m = p.month_name()?;
        p.skip_spaces();
        let yy = p.num(2, 4)? as i32;
        let y = if yy < 50 {
            2000 + yy
        } else if yy < 1000 {
            1900 + yy
        } else {
            yy
        };
        p.skip_spaces();
        let hh = p.num(2, 2)? as u8;
        let mm = p.eat(b':').then(|| p.num(2, 2))?? as u8;
        let ss = if p.eat(b':') { p.num(2, 2)? as u8 } else { 0 };
        p.skip_spaces();
        let offset = p.offset()?;
        if d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 60 {
            return None;
        }
        Some(C::datetime_to_secs((y, m, d, hh, mm, ss.min(59))) - offset as i64)
    };
    parse().map_or(1, |t| t as u64)
}

#[inline(never)]
fn imap_parse<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    // "11-Oct-2026 14:34:56 +0200", with a space before one-digit days
    let parse = || {
        let mut p = Scan::new(s);
        p.skip_spaces();
        let d = p.num(1, 2)? as u8;
        let m = p.eat(b'-').then(|| p.month_name())??;
        let y = p.eat(b'-').then(|| p.num(4, 4))?? as i32;
        p.skip_spaces();
        let hh = p.num(2, 2)? as u8;
        let mm = p.eat(b':').then(|| p.num(2, 2))?? as u8;
        let ss = p.eat(b':').then(|| p.num(2, 2))?? as u8;
        p.skip_spaces();
        let offset = p.offset()?;
        if d < 1 || d > C::days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
            return None;
        }
        Some(C::datetime_to_secs((y, m, d, hh, mm, ss)) - offset as i64)
    };
    parse().map_or(1, |t| t as u64)
}

#[inline(never)]
fn jwt<C: Calendar>(exp: i64, iat: i64, nbf: i64, now: i64) -> u64 {
    black_box(C::SALT);
    // openidconnect turns each claim into a date-time before comparing
    let exp_dt = C::secs_to_datetime(exp);
    let iat_dt = C::secs_to_datetime(iat);
    let nbf_dt = C::secs_to_datetime(nbf);
    let valid = exp > now && nbf <= now && iat <= now;
    text::mix(text::mix(dt_hash(exp_dt), dt_hash(iat_dt)), dt_hash(nbf_dt)) ^ valid as u64
}

fn dt_hash((y, m, d, hh, mm, ss): DateTime) -> u64 {
    (y as u64) << 40 | (m as u64) << 32 | (d as u64) << 24 | (hh as u64) << 16 | (mm as u64) << 8 | ss as u64
}

// Inputs

/// A timestamp of a value in a message: mostly near the present, some in the
/// past years, and rarely anywhere in 0001-9999
fn value_time(rng: &mut fastrand::Rng) -> i64 {
    match rng.u32(0..100) {
        0..=69 => near_now(rng, 30 * DAY),
        70..=97 => NOW - rng.i64(0..20 * YEAR),
        _ => rng.i64(-62_135_596_800..=253_402_300_799),
    }
}

/// Make about one in fifty dates invalid, as the 31st of a 30-day month
fn invalidate(rng: &mut fastrand::Rng, s: String) -> String {
    if rng.u32(0..50) != 0 {
        return s;
    }
    // Replace the first day field found by the 31st of the month or the 30th of
    // February: valid strings have the day after a month, which this does not
    // try to find exactly, so it may produce other invalid strings too
    s.replacen("-01", "-31", 1).replacen(" 01 ", " 31 ", 1).replacen("02-2", "02-3", 1)
}

fn fields(t: i64) -> (i32, u8, u8, u8, u8, u8, u8) {
    let (y, m, d, hh, mm, ss) = datealgo::secs_to_datetime(t);
    (y, m, d, hh, mm, ss, datealgo::date_to_weekday((y, m, d)))
}

fn rfc3339(t: i64, nanos: u32, digits: u32, offset: i32) -> String {
    let (y, m, d, hh, mm, ss, _) = fields(t + offset as i64);
    let mut s = format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}");
    if digits > 0 {
        s += &format!(".{:0w$}", nanos / 10u32.pow(9 - digits), w = digits as usize);
    }
    if offset == 0 {
        s.push('Z');
    } else {
        let o = offset.unsigned_abs();
        s += &format!("{}{:02}:{:02}", if offset < 0 { '-' } else { '+' }, o / 3600, o / 60 % 60);
    }
    s
}

fn name(s: &[u8; 3]) -> &str {
    std::str::from_utf8(s).unwrap()
}

fn http_date(t: i64) -> String {
    let (y, m, d, hh, mm, ss, wd) = fields(t);
    format!(
        "{}, {d:02} {} {y} {hh:02}:{mm:02}:{ss:02} GMT",
        name(WEEKDAYS[wd as usize - 1]),
        name(MONTHS[m as usize - 1])
    )
}

fn rfc850(t: i64) -> String {
    let (y, m, d, hh, mm, ss, wd) = fields(t);
    format!(
        "{}, {d:02}-{}-{:02} {hh:02}:{mm:02}:{ss:02} GMT",
        text::WEEKDAY_NAMES[wd as usize - 1],
        name(MONTHS[m as usize - 1]),
        y % 100
    )
}

fn asctime(t: i64) -> String {
    let (y, m, d, hh, mm, ss, wd) = fields(t);
    format!(
        "{} {} {d:>2} {hh:02}:{mm:02}:{ss:02} {y}",
        name(WEEKDAYS[wd as usize - 1]),
        name(MONTHS[m as usize - 1])
    )
}

fn email_date(t: i64, offset: i32, weekday: bool) -> String {
    let (y, m, d, hh, mm, ss, wd) = fields(t + offset as i64);
    let o = offset.unsigned_abs();
    format!(
        "{}{d} {} {y} {hh:02}:{mm:02}:{ss:02} {}{:02}{:02}",
        if weekday {
            format!("{}, ", name(WEEKDAYS[wd as usize - 1]))
        } else {
            String::new()
        },
        name(MONTHS[m as usize - 1]),
        if offset < 0 { '-' } else { '+' },
        o / 3600,
        o / 60 % 60
    )
}

fn imap_date(t: i64, offset: i32) -> String {
    let (y, m, d, hh, mm, ss, _) = fields(t + offset as i64);
    let o = offset.unsigned_abs();
    format!(
        "{d:>2}-{}-{y} {hh:02}:{mm:02}:{ss:02} {}{:02}{:02}",
        name(MONTHS[m as usize - 1]),
        if offset < 0 { '-' } else { '+' },
        o / 3600,
        o / 60 % 60
    )
}
