//! Calendar arithmetic and business rules: adding months and years with
//! clamping, ages and days between dates, the bounds of months, quarters and
//! years, business days that skip weekends and holidays, holiday rules,
//! relative dates as natural-language parsers produce them, and calendar
//! month views.

use super::text;
use super::*;
use std::hint::black_box;

#[derive(Clone, Copy)]
enum Event {
    /// Add months, clamping the day to the end of the month, as chronoutil and
    /// chrono's `checked_add_months` do for billing cycles and subscriptions
    AddMonths(Date, i32),
    /// Subtract months as git does for "N months ago", letting the day roll
    /// over into the next month
    MonthsAgo(Date, i32),
    /// Add years for an anniversary, with February 29 becoming February 28
    AddYears(Date, i32),
    /// Age in whole years at a date, and the days between the dates
    Age(Date, Date),
    /// The first and last day of the month, quarter or year of a date
    Bounds(Date, u8),
    /// Advance a number of business days, skipping weekends and holidays
    BusinessDays(Date, u8),
    /// The holidays of a year
    Holidays(i32),
    /// The next given day of the week after a date ("next friday")
    NextWeekday(Date, u8),
    /// The days of a month's calendar view, starting on Monday
    MonthView(i32, u8),
}

pub struct Arith {
    events: Vec<Event>,
}

impl Workload for Arith {
    const NAME: &'static str = "arith";
    const WEIGHT: u32 = 10;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let date = |rng: &mut fastrand::Rng| {
            let (y, m, d) = datealgo::rd_to_date((near_now(rng, 10 * YEAR) / DAY) as i32);
            // Days at the end of months are common, as billing dates are
            if rng.u32(0..4) == 0 {
                (y, m, datealgo::days_in_month(y, m))
            } else {
                (y, m, d)
            }
        };
        let events = (0..EVENTS)
            .map(
                |_| match pick(rng, &[(0, 18), (1, 5), (2, 7), (3, 12), (4, 12), (5, 14), (6, 5), (7, 12), (8, 15)]) {
                    0 => Event::AddMonths(date(rng), {
                        let c = [(1, 50), (3, 15), (6, 10), (12, 15), (rng.i32(-36..=36), 10)];
                        pick(rng, &c)
                    }),
                    1 => Event::MonthsAgo(date(rng), rng.i32(1..=24)),
                    2 => Event::AddYears(if rng.u32(0..10) == 0 { (2024, 2, 29) } else { date(rng) }, rng.i32(1..=10)),
                    3 => Event::Age(datealgo::rd_to_date((NOW / DAY - rng.i64(0..36_500)) as i32), date(rng)),
                    4 => Event::Bounds(date(rng), rng.u8(0..3)),
                    5 => Event::BusinessDays(date(rng), {
                        let c = [(1, 30), (2, 20), (5, 20), (10, 15), (rng.u8(1..=60), 15)];
                        pick(rng, &c)
                    }),
                    6 => Event::Holidays(datealgo::rd_to_date((near_now(rng, 10 * YEAR) / DAY) as i32).0),
                    7 => Event::NextWeekday(date(rng), rng.u8(1..=7)),
                    _ => {
                        let (y, m, _) = date(rng);
                        Event::MonthView(y, m)
                    }
                },
            )
            .collect();
        Arith { events }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let dep = dep as u8;
        let dd = |(y, m, d): Date| (y, m, d + dep);
        match self.events[i % EVENTS] {
            Event::AddMonths(d, n) => add_months::<C>(dd(d), n),
            Event::MonthsAgo(d, n) => months_ago::<C>(dd(d), n),
            Event::AddYears(d, n) => add_years::<C>(dd(d), n),
            Event::Age(born, at) => age::<C>(dd(born), at),
            Event::Bounds(d, unit) => bounds::<C>(dd(d), unit),
            Event::BusinessDays(d, n) => business_days::<C>(dd(d), n),
            Event::Holidays(y) => holidays::<C>(y + dep as i32),
            Event::NextWeekday(d, wd) => next_weekday::<C>(dd(d), wd),
            Event::MonthView(y, m) => month_view::<C>(y, m + dep),
        }
    }
}

fn date_bits((y, m, d): Date) -> u64 {
    (y as u64) << 16 | (m as u64) << 8 | d as u64
}

#[inline(never)]
fn add_months<C: Calendar>((y, m, d): Date, n: i32) -> u64 {
    black_box(C::SALT);
    let total = y * 12 + m as i32 - 1 + n;
    let (y, m) = (total.div_euclid(12), total.rem_euclid(12) as u8 + 1);
    let d = d.min(C::days_in_month(y, m));
    // The renewal as a timestamp at midnight
    C::datetime_to_secs((y, m, d, 0, 0, 0)) as u64
}

#[inline(never)]
fn months_ago<C: Calendar>((y, m, d): Date, n: i32) -> u64 {
    black_box(C::SALT);
    let total = y * 12 + m as i32 - 1 - n;
    let first = (total.div_euclid(12), total.rem_euclid(12) as u8 + 1, 1);
    date_bits(C::rd_to_date(C::date_to_rd(first) + d as i32 - 1))
}

#[inline(never)]
fn add_years<C: Calendar>((y, m, d): Date, n: i32) -> u64 {
    black_box(C::SALT);
    let y = y + n;
    let d = if m == 2 && d == 29 && !C::is_leap_year(y) { 28 } else { d };
    C::date_to_rd((y, m, d)) as u64
}

#[inline(never)]
fn age<C: Calendar>(born: Date, at: Date) -> u64 {
    black_box(C::SALT);
    let years = at.0 - born.0 - ((at.1, at.2) < (born.1, born.2)) as i32;
    let days = C::date_to_rd(at) - C::date_to_rd(born);
    (years as u64) << 32 | days as u32 as u64
}

#[inline(never)]
fn bounds<C: Calendar>((y, m, _): Date, unit: u8) -> u64 {
    black_box(C::SALT);
    let (first, last) = match unit {
        0 => ((y, m, 1), (y, m, C::days_in_month(y, m))),
        1 => {
            let q = (m - 1) / 3 * 3 + 1;
            ((y, q, 1), (y, q + 2, C::days_in_month(y, q + 2)))
        }
        _ => ((y, 1, 1), (y, 12, 31)),
    };
    let (a, b) = (C::date_to_rd(first), C::date_to_rd(last));
    (a as u64) << 32 | (b - a) as u32 as u64
}

/// The nth (from 1) or, for 0, the last given day of the week in a month
fn nth_weekday<C: Calendar>(y: i32, m: u8, n: u8, wd: u8) -> i32 {
    if n == 0 {
        let last = C::date_to_rd((y, m, C::days_in_month(y, m)));
        last - (C::rd_to_weekday(last) as i32 - wd as i32).rem_euclid(7)
    } else {
        let first = C::date_to_rd((y, m, 1));
        first + (wd as i32 - C::rd_to_weekday(first) as i32).rem_euclid(7) + (n as i32 - 1) * 7
    }
}

/// A fixed holiday moved to Friday or Monday when it falls on a weekend
fn observed<C: Calendar>(y: i32, m: u8, d: u8) -> i32 {
    let n = C::date_to_rd((y, m, d));
    match C::rd_to_weekday(n) {
        6 => n - 1,
        7 => n + 1,
        _ => n,
    }
}

/// Easter Sunday by the anonymous Gregorian algorithm
fn easter<C: Calendar>(y: i32) -> i32 {
    let (a, b, c) = (y % 19, y / 100, y % 100);
    let (d, e) = (b / 4, b % 4);
    let g = (8 * b + 13) / 25;
    let h = (19 * a + b - d - g + 15) % 30;
    let (i, k) = (c / 4, c % 4);
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let mm = (a + 11 * h + 19 * l) / 433;
    let month = (h + l - 7 * mm + 90) / 25;
    let day = (h + l - 7 * mm + 33 * month + 19) % 32;
    C::date_to_rd((y, month as u8, day as u8))
}

/// Whether a day is a holiday, recomputing the rules of its month, as bdays
/// and holiday calendars do
fn is_holiday<C: Calendar>(n: i32) -> bool {
    let (y, m, _) = C::rd_to_date(n);
    match m {
        1 => n == observed::<C>(y, 1, 1) || n == nth_weekday::<C>(y, 1, 3, 1),
        2 => n == nth_weekday::<C>(y, 2, 3, 1),
        3 | 4 => n == easter::<C>(y) - 2,
        5 => n == nth_weekday::<C>(y, 5, 0, 1),
        6 => n == observed::<C>(y, 6, 19),
        7 => n == observed::<C>(y, 7, 4),
        9 => n == nth_weekday::<C>(y, 9, 1, 1),
        11 => n == nth_weekday::<C>(y, 11, 4, 4),
        12 => n == observed::<C>(y, 12, 25) || n == observed::<C>(y + 1, 1, 1),
        _ => false,
    }
}

#[inline(never)]
fn business_days<C: Calendar>(d: Date, n: u8) -> u64 {
    black_box(C::SALT);
    // Step a day at a time, as bdays does
    let mut d = d;
    let mut left = n;
    while left > 0 {
        d = C::next_date(d);
        if C::date_to_weekday(d) < 6 && !is_holiday::<C>(C::date_to_rd(d)) {
            left -= 1;
        }
    }
    date_bits(d)
}

#[inline(never)]
fn holidays<C: Calendar>(y: i32) -> u64 {
    black_box(C::SALT);
    let days = [
        observed::<C>(y, 1, 1),
        nth_weekday::<C>(y, 1, 3, 1),
        nth_weekday::<C>(y, 2, 3, 1),
        easter::<C>(y) - 2,
        nth_weekday::<C>(y, 5, 0, 1),
        observed::<C>(y, 6, 19),
        observed::<C>(y, 7, 4),
        nth_weekday::<C>(y, 9, 1, 1),
        nth_weekday::<C>(y, 11, 4, 4),
        observed::<C>(y, 12, 25),
    ];
    days.iter().fold(0, |h, &n| text::mix(h, date_bits(C::rd_to_date(n))))
}

#[inline(never)]
fn next_weekday<C: Calendar>(d: Date, wd: u8) -> u64 {
    black_box(C::SALT);
    let n = C::date_to_rd(d);
    let ahead = (wd as i32 - C::date_to_weekday(d) as i32 - 1).rem_euclid(7) + 1;
    date_bits(C::rd_to_date(n + ahead))
}

#[inline(never)]
fn month_view<C: Calendar>(y: i32, m: u8) -> u64 {
    black_box(C::SALT);
    // The view starts with the end of the previous month up to Monday, then
    // the month, then the start of the next month up to Sunday
    let first = (y, m, 1);
    let lead = C::date_to_weekday(first) - 1;
    let mut day = first;
    for _ in 0..lead {
        day = C::prev_date(day);
    }
    let cells = (lead as u32 + C::days_in_month(y, m) as u32).div_ceil(7) * 7;
    let mut h = 0;
    for _ in 0..cells {
        h = text::mix(h, date_bits(day));
        day = C::next_date(day);
    }
    h
}
