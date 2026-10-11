//! Weeks: ISO week numbers for grouping and reports, ISO week dates as text,
//! the range of a week, the number of weeks in a year for week pickers, US
//! week numbers as strftime's %U gives them, weeks of a month, and RFC 5545
//! week numbers with a configurable start of the week.

use super::text::{self, Buf, Scan};
use super::*;
use std::hint::black_box;

#[derive(Clone, Copy)]
enum Event {
    /// The ISO week of a date, for grouping
    IsoWeek(Date),
    /// The ISO week of a timestamp
    IsoWeekOfTime(i64),
    /// A date as an ISO week date "2026-W41-6"
    FormatIsoWeek(Date),
    /// "2026-W41-6" or "2026W41" to a date, checking the week exists
    ParseIsoWeek(u32),
    /// The Monday and Sunday of the week of a date
    WeekRange(Date),
    /// The weeks in this, the previous and the next year
    WeeksInYear(i32),
    /// The week of the year with weeks starting on Sunday, as %U
    UsWeek(Date),
    /// The first day of week `n` of a year with weeks starting on day `wkst`,
    /// as RFC 5545 BYWEEKNO counts them
    RruleWeek(i32, u8, u8),
    /// The week of the month of a date
    WeekOfMonth(Date),
}

pub struct Weeks {
    events: Vec<Event>,
    strings: Vec<String>,
}

impl Workload for Weeks {
    const NAME: &'static str = "weeks";
    const WEIGHT: u32 = 5;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let date = |rng: &mut fastrand::Rng| {
            // A quarter of the dates are near the turn of a year, where ISO
            // weeks belong to another year
            let n = (near_now(rng, 5 * YEAR) / DAY) as i32;
            if rng.u32(0..4) == 0 {
                let (y, _, _) = datealgo::rd_to_date(n);
                datealgo::rd_to_date(datealgo::date_to_rd((y, 1, 1)) + rng.i32(-7..7))
            } else {
                datealgo::rd_to_date(n)
            }
        };
        let mut strings = Vec::new();
        let events = (0..EVENTS)
            .map(
                |_| match pick(rng, &[(0, 25), (1, 10), (2, 10), (3, 10), (4, 12), (5, 5), (6, 8), (7, 8), (8, 12)]) {
                    0 => Event::IsoWeek(date(rng)),
                    1 => Event::IsoWeekOfTime(near_now(rng, 2 * YEAR)),
                    2 => Event::FormatIsoWeek(date(rng)),
                    3 => {
                        let (y, w, d) = datealgo::date_to_isoweekdate(date(rng));
                        // Some ask for week 53 of years that have none
                        let w = if rng.u32(0..20) == 0 { 53 } else { w };
                        strings.push(if rng.bool() {
                            format!("{y:04}-W{w:02}-{d}")
                        } else {
                            format!("{y:04}W{w:02}")
                        });
                        Event::ParseIsoWeek(strings.len() as u32 - 1)
                    }
                    4 => Event::WeekRange(date(rng)),
                    5 => Event::WeeksInYear(date(rng).0),
                    6 => Event::UsWeek(date(rng)),
                    7 => Event::RruleWeek(
                        date(rng).0,
                        {
                            let c = [(1, 70), (7, 25), (rng.u8(1..=7), 5)];
                            pick(rng, &c)
                        },
                        rng.u8(1..=52),
                    ),
                    _ => Event::WeekOfMonth(date(rng)),
                },
            )
            .collect();
        Weeks { events, strings }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let dd = |(y, m, d): Date| (y, m, d + dep as u8);
        match self.events[i % EVENTS] {
            Event::IsoWeek(d) => iso_week::<C>(dd(d)),
            Event::IsoWeekOfTime(t) => iso_week_of_time::<C>(t + dep as i64),
            Event::FormatIsoWeek(d) => format_iso_week::<C>(dd(d)),
            Event::ParseIsoWeek(k) => parse_iso_week::<C>(self.strings[k as usize + dep as usize].as_bytes()),
            Event::WeekRange(d) => week_range::<C>(dd(d)),
            Event::WeeksInYear(y) => weeks_in_year::<C>(y + dep as i32),
            Event::UsWeek(d) => us_week::<C>(dd(d)),
            Event::RruleWeek(y, wkst, n) => rrule_week::<C>(y + dep as i32, wkst, n),
            Event::WeekOfMonth(d) => week_of_month::<C>(dd(d)),
        }
    }
}

fn iwd_bits((y, w, d): IsoWeekDate) -> u64 {
    (y as u64) << 16 | (w as u64) << 8 | d as u64
}

#[inline(never)]
fn iso_week<C: Calendar>(d: Date) -> u64 {
    black_box(C::SALT);
    iwd_bits(C::date_to_isoweekdate(d))
}

#[inline(never)]
fn iso_week_of_time<C: Calendar>(t: i64) -> u64 {
    black_box(C::SALT);
    let (days, _, _, _) = C::secs_to_dhms(t);
    iwd_bits(C::rd_to_isoweekdate(days))
}

#[inline(never)]
fn format_iso_week<C: Calendar>(d: Date) -> u64 {
    black_box(C::SALT);
    let (y, w, wd) = C::date_to_isoweekdate(d);
    let mut b = Buf::<12>::new();
    b.put4(y as u32);
    b.extend(b"-W");
    b.put2(w as u32);
    b.push(b'-');
    b.push(b'0' + wd);
    text::hash(b.as_bytes())
}

#[inline(never)]
fn parse_iso_week<C: Calendar>(s: &[u8]) -> u64 {
    black_box(C::SALT);
    let parse = || {
        let mut p = Scan::new(s);
        let y = p.num(4, 4)? as i32;
        p.eat(b'-');
        if !p.eat(b'W') {
            return None;
        }
        let w = p.num(2, 2)? as u8;
        let d = if p.eat(b'-') { p.num(1, 1)? as u8 } else { 1 };
        if w < 1 || w > C::isoweeks_in_year(y) || !(1..=7).contains(&d) {
            return None;
        }
        Some(C::isoweekdate_to_date((y, w, d)))
    };
    parse().map_or(1, |(y, m, d)| (y as u64) << 16 | (m as u64) << 8 | d as u64)
}

#[inline(never)]
fn week_range<C: Calendar>(d: Date) -> u64 {
    black_box(C::SALT);
    let n = C::date_to_rd(d);
    let monday = n - (C::rd_to_weekday(n) as i32 - 1);
    let (a, b) = (C::rd_to_date(monday), C::rd_to_date(monday + 6));
    text::mix(
        (a.0 as u64) << 16 | (a.1 as u64) << 8 | a.2 as u64,
        (b.0 as u64) << 16 | (b.1 as u64) << 8 | b.2 as u64,
    )
}

#[inline(never)]
fn weeks_in_year<C: Calendar>(y: i32) -> u64 {
    black_box(C::SALT);
    (C::isoweeks_in_year(y - 1) as u64) << 16 | (C::isoweeks_in_year(y) as u64) << 8 | C::isoweeks_in_year(y + 1) as u64
}

#[inline(never)]
fn us_week<C: Calendar>((y, m, d): Date) -> u64 {
    black_box(C::SALT);
    let n = C::date_to_rd((y, m, d));
    let yday = n - C::date_to_rd((y, 1, 1));
    let sunday0 = C::rd_to_weekday(n) as i32 % 7;
    ((yday + 7 - sunday0) / 7) as u64
}

#[inline(never)]
fn rrule_week<C: Calendar>(y: i32, wkst: u8, n: u8) -> u64 {
    black_box(C::SALT);
    // Week 1 is the first week with at least four days of the year, with weeks
    // starting on `wkst`, as dateutil and the rrule crate compute it
    let jan1 = C::date_to_rd((y, 1, 1));
    let start = (7 - C::rd_to_weekday(jan1) as i32 + wkst as i32).rem_euclid(7);
    let week1 = if start >= 4 { jan1 + start - 7 } else { jan1 + start };
    let (y, m, d) = C::rd_to_date(week1 + (n as i32 - 1) * 7);
    (y as u64) << 16 | (m as u64) << 8 | d as u64
}

#[inline(never)]
fn week_of_month<C: Calendar>((y, m, d): Date) -> u64 {
    black_box(C::SALT);
    let first = C::date_to_weekday((y, m, 1)) as u32;
    ((d as u32 + first - 2) / 7 + 1) as u64
}
