//! Logging and tracing: every line formats the present, in the formats of
//! tracing-subscriber, env_logger, flexi_logger, syslog, slog-term and web
//! server access logs, and rolling file writers check whether to start a new
//! file, as tracing-appender does. Loggers do not cache any of this.

use super::text::{Buf, MONTHS};
use super::*;
use std::{
    fmt::Write,
    hint::black_box,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy)]
enum Event {
    /// tracing-subscriber: "2026-10-11T12:34:56.123456Z" from a SystemTime
    Rfc3339Micros(SystemTime),
    /// env_logger: "2026-10-11T12:34:56.123Z" with `write!`
    Rfc3339Millis(i64, u32),
    /// flexi_logger: "2026-10-11 14:34:56.123456 +02:00" in local time
    Local(i64, u32, i32),
    /// RFC 3164 syslog: "Oct 11 14:34:56" in local time
    Syslog(i64, i32),
    /// slog-term: "Oct 11 14:34:56.123" in local time, from a SystemTime
    SlogTerm(SystemTime, i32),
    /// Access logs: "[11/Oct/2026:14:34:56 +0200]"
    AccessLog(i64, i32),
    /// A rolling file writer: the time of a write, the start of the period of
    /// the current file, and the period
    Roll(SystemTime, i64, Period),
}

#[derive(Clone, Copy)]
enum Period {
    Hourly,
    Daily,
    Weekly,
}

pub struct Logging {
    events: Vec<Event>,
}

impl Workload for Logging {
    const NAME: &'static str = "logging";
    const WEIGHT: u32 = 20;

    fn new(rng: &mut fastrand::Rng) -> Self {
        // Log lines of a few days, a few milliseconds apart on average, with
        // bursts in the same second, in a few local time zones
        let mut t = (NOW - 3 * DAY) as i128 * 1_000_000_000;
        let offsets = [0, 3600, 7200, 19800, -18000, -28800];
        let events = (0..EVENTS)
            .map(|_| {
                t += if rng.u32(0..10) < 3 {
                    rng.i128(0..1000)
                } else {
                    rng.i128(0..12_000_000)
                };
                // Some lines are of other processes running for longer
                let nanos = if rng.u32(0..20) == 0 {
                    t - rng.i128(0..90 * 86_400_000_000_000)
                } else {
                    t
                };
                let (secs, sub) = (nanos.div_euclid(1_000_000_000) as i64, nanos.rem_euclid(1_000_000_000) as u32);
                let st = UNIX_EPOCH + Duration::new(secs as u64, sub);
                let offset = offsets[rng.usize(..offsets.len())];
                match pick(rng, &[(0, 25), (1, 20), (2, 15), (3, 10), (4, 8), (5, 10), (6, 12)]) {
                    0 => Event::Rfc3339Micros(st),
                    1 => Event::Rfc3339Millis(secs, sub),
                    2 => Event::Local(secs, sub, offset),
                    3 => Event::Syslog(secs, offset),
                    4 => Event::SlogTerm(st, offset),
                    5 => Event::AccessLog(secs, offset),
                    _ => {
                        let period = pick(rng, &[(Period::Hourly, 3), (Period::Daily, 6), (Period::Weekly, 1)]);
                        // The current file usually started recently, and
                        // sometimes the period has just ended
                        let len = match period {
                            Period::Hourly => 3600,
                            Period::Daily => DAY,
                            Period::Weekly => 7 * DAY,
                        };
                        let start = if rng.u32(0..10) == 0 {
                            secs - len - rng.i64(0..len)
                        } else {
                            secs - rng.i64(0..len)
                        };
                        Event::Roll(st, start, period)
                    }
                }
            })
            .collect();
        Logging { events }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let d = Duration::from_nanos(dep);
        match self.events[i % EVENTS] {
            Event::Rfc3339Micros(st) => rfc3339_micros::<C>(st + d),
            Event::Rfc3339Millis(secs, nanos) => rfc3339_millis::<C>(secs + dep as i64, nanos),
            Event::Local(secs, nanos, offset) => local::<C>(secs + dep as i64, nanos, offset),
            Event::Syslog(secs, offset) => syslog::<C>(secs + dep as i64, offset),
            Event::SlogTerm(st, offset) => slog_term::<C>(st + d, offset),
            Event::AccessLog(secs, offset) => access_log::<C>(secs + dep as i64, offset),
            Event::Roll(st, start, period) => roll::<C>(st + d, start, period),
        }
    }
}

#[inline(never)]
fn rfc3339_micros<C: Calendar>(st: SystemTime) -> u64 {
    black_box(C::SALT);
    let Some((y, m, d, hh, mm, ss, nanos)) = C::systemtime_to_datetime(st) else {
        return 0;
    };
    let mut b = Buf::<32>::new();
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
    b.put_frac(nanos, 6);
    b.push(b'Z');
    text::hash(b.as_bytes())
}

#[inline(never)]
fn rfc3339_millis<C: Calendar>(secs: i64, nanos: u32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs);
    let mut b = Buf::<32>::new();
    let _ = write!(b, "{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{:03}Z", nanos / 1_000_000);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn local<C: Calendar>(secs: i64, nanos: u32, offset: i32) -> u64 {
    black_box(C::SALT);
    let (days, hh, mm, ss) = C::secs_to_dhms(secs + offset as i64);
    let (y, m, d) = C::rd_to_date(days);
    let mut b = Buf::<40>::new();
    b.put_div(y as u64, 4);
    b.push(b'-');
    b.put_div(m as u64, 2);
    b.push(b'-');
    b.put_div(d as u64, 2);
    b.push(b' ');
    b.put_div(hh as u64, 2);
    b.push(b':');
    b.put_div(mm as u64, 2);
    b.push(b':');
    b.put_div(ss as u64, 2);
    b.put_frac(nanos, 6);
    b.push(b' ');
    b.push(if offset < 0 { b'-' } else { b'+' });
    let o = offset.unsigned_abs();
    b.put_div((o / 3600) as u64, 2);
    b.push(b':');
    b.put_div((o / 60 % 60) as u64, 2);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn syslog<C: Calendar>(secs: i64, offset: i32) -> u64 {
    black_box(C::SALT);
    let (_, m, d, hh, mm, ss) = C::secs_to_datetime(secs + offset as i64);
    let mut b = Buf::<16>::new();
    b.extend(MONTHS[m as usize - 1]);
    b.push(b' ');
    if d < 10 {
        b.push(b' ');
        b.push(b'0' + d);
    } else {
        b.put2(d as u32);
    }
    b.push(b' ');
    b.put2(hh as u32);
    b.push(b':');
    b.put2(mm as u32);
    b.push(b':');
    b.put2(ss as u32);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn slog_term<C: Calendar>(st: SystemTime, offset: i32) -> u64 {
    black_box(C::SALT);
    // slog-term converts the local time through a second library, so the
    // time goes from SystemTime to seconds and then to a date and time
    let Some((secs, nanos)) = C::systemtime_to_secs(st) else {
        return 0;
    };
    let (_, m, d, hh, mm, ss) = C::secs_to_datetime(secs + offset as i64);
    let mut b = Buf::<24>::new();
    let _ = write!(
        b,
        "{} {d:02} {hh:02}:{mm:02}:{ss:02}.{:03}",
        std::str::from_utf8(MONTHS[m as usize - 1]).unwrap_or(""),
        nanos / 1_000_000
    );
    text::hash(b.as_bytes())
}

#[inline(never)]
fn access_log<C: Calendar>(secs: i64, offset: i32) -> u64 {
    black_box(C::SALT);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(secs + offset as i64);
    let mut b = Buf::<32>::new();
    b.push(b'[');
    b.put2(d as u32);
    b.push(b'/');
    b.extend(MONTHS[m as usize - 1]);
    b.push(b'/');
    b.put4(y as u32);
    b.push(b':');
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
    b.push(b']');
    text::hash(b.as_bytes())
}

#[inline(never)]
fn roll<C: Calendar>(st: SystemTime, start: i64, period: Period) -> u64 {
    black_box(C::SALT);
    let Some((now, _)) = C::systemtime_to_secs(st) else {
        return 0;
    };
    // The end of the period of the current file, from its start
    let (days, hh, _, _) = C::secs_to_dhms(start);
    let end = match period {
        Period::Hourly => C::dhms_to_secs((days, hh, 0, 0)) + 3600,
        Period::Daily => C::dhms_to_secs((days, 0, 0, 0)) + DAY,
        Period::Weekly => C::dhms_to_secs((days - C::rd_to_weekday(days) as i32 % 7 + 7, 0, 0, 0)),
    };
    if now < end {
        return end as u64;
    }
    // Start a new file named by the new period: "app.2026-10-11-13"
    let (days, hh, _, _) = C::secs_to_dhms(now);
    let (y, m, d) = match period {
        Period::Weekly => C::rd_to_date(days - C::rd_to_weekday(days) as i32 % 7),
        _ => C::rd_to_date(days),
    };
    let mut b = Buf::<24>::new();
    b.extend(b"app.");
    b.put4(y as u32);
    b.push(b'-');
    b.put2(m as u32);
    b.push(b'-');
    b.put2(d as u32);
    if let Period::Hourly = period {
        b.push(b'-');
        b.put2(hh as u32);
    }
    text::hash(b.as_bytes())
}
