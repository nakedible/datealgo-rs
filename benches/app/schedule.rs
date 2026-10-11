//! Scheduling and time zones: cron engines finding the next run of a job, as
//! the cron, croner and saffron crates do, RFC 5545 recurrences expanded into
//! occurrences as the rrule crate does, and conversions between UTC and local
//! time with POSIX time zone rules such as "M3.2.0", as zone libraries do past
//! their last transition.

use super::text;
use super::*;
use std::hint::black_box;

/// A cron schedule: the allowed minutes, hours, days of the month, months and
/// days of the week (Sunday is 0), as bit sets, with `L` for the last day of
/// the month and `#n` for the nth given day of the week
#[derive(Clone, Copy)]
struct Cron {
    minutes: u64,
    hours: u32,
    days: u32,
    months: u16,
    weekdays: u8,
    last_day: bool,
    nth: Option<(u8, u8)>,
}

const ALL_DAYS: u32 = !1;
const ALL_WEEKDAYS: u8 = 0x7f;

const fn bits(from: u32, to: u32, step: u32) -> u64 {
    let mut b = 0;
    let mut i = from;
    while i <= to {
        b |= 1 << i;
        i += step;
    }
    b
}

const fn cron(minutes: u64, hours: u64, days: u64, months: u64, weekdays: u64) -> Cron {
    Cron {
        minutes,
        hours: hours as u32,
        days: days as u32,
        months: months as u16,
        weekdays: weekdays as u8,
        last_day: false,
        nth: None,
    }
}

/// Schedules like those in real crontabs and job schedulers
const CRONS: [Cron; 13] = [
    cron(bits(0, 59, 5), bits(0, 23, 1), ALL_DAYS as u64, bits(1, 12, 1), ALL_WEEKDAYS as u64), // */5 * * * *
    cron(1, bits(0, 23, 1), ALL_DAYS as u64, bits(1, 12, 1), ALL_WEEKDAYS as u64),              // 0 * * * *
    cron(1, 1, ALL_DAYS as u64, bits(1, 12, 1), ALL_WEEKDAYS as u64),                           // 0 0 * * *
    cron(1 << 30, 1 << 2, ALL_DAYS as u64, bits(1, 12, 1), bits(1, 5, 1)),                      // 30 2 * * 1-5
    cron(1, 1 << 9, ALL_DAYS as u64, bits(1, 12, 1), 1 << 1),                                   // 0 9 * * MON
    cron(1, 1, 1 << 1, bits(1, 12, 1), ALL_WEEKDAYS as u64),                                    // 0 0 1 * *
    Cron {
        last_day: true,
        ..cron(1, 1, 0, bits(1, 12, 1), ALL_WEEKDAYS as u64)
    }, // 0 0 L * *
    Cron {
        nth: Some((5, 3)),
        ..cron(1, 1, ALL_DAYS as u64, bits(1, 12, 1), 0)
    }, // 0 0 * * 5#3
    cron(1, 1, 1 << 29, 1 << 2, ALL_WEEKDAYS as u64),                                           // 0 0 29 2 *
    cron(1, 1, 1 << 13, bits(1, 12, 1), 1 << 5),                                                // 0 0 13 * 5
    cron(1 << 15, 1 << 10, ALL_DAYS as u64, bits(1, 12, 1), 1 << 6 | 1),                        // 15 10 * * 6,0
    cron(1, bits(0, 23, 6), ALL_DAYS as u64, bits(1, 12, 1), ALL_WEEKDAYS as u64),              // 0 */6 * * *
    cron(1, 1, 1 << 1, 1 << 1, ALL_WEEKDAYS as u64),                                            // 0 0 1 1 *
];

/// A recurrence rule
#[derive(Clone, Copy)]
enum Rrule {
    /// MONTHLY;BYDAY=nWD, with n of -1 for the last
    MonthlyNth(i8, u8),
    /// MONTHLY;BYMONTHDAY=d, skipping months without the day
    MonthlyDay(u8),
    /// YEARLY;BYMONTH=m;BYDAY=nWD
    YearlyNth(u8, i8, u8),
    /// WEEKLY;BYDAY=days, as a bit set of days of the week, Monday at 1
    Weekly(u8),
    /// DAILY;INTERVAL=n
    Daily(u8),
}

/// A POSIX TZ rule: standard offset, DST offset (0 for none), and the start
/// and end of DST as (month, week 1-5 with 5 the last, weekday with Sunday 0,
/// hour)
#[derive(Clone, Copy)]
struct Zone {
    std: i32,
    dst: i32,
    start: (u8, u8, u8, u8),
    end: (u8, u8, u8, u8),
}

/// EU, US, Australian Eastern and Indian time
const ZONES: [Zone; 4] = [
    Zone {
        std: 3600,
        dst: 7200,
        start: (3, 5, 0, 1),
        end: (10, 5, 0, 1),
    },
    Zone {
        std: -18000,
        dst: -14400,
        start: (3, 2, 0, 2),
        end: (11, 1, 0, 2),
    },
    Zone {
        std: 36000,
        dst: 39600,
        start: (10, 1, 0, 2),
        end: (4, 1, 0, 3),
    },
    Zone {
        std: 19800,
        dst: 0,
        start: (0, 0, 0, 0),
        end: (0, 0, 0, 0),
    },
];

#[derive(Clone, Copy)]
enum Event {
    /// The next run of a cron schedule after a time
    CronNext(u8, i64),
    /// A number of occurrences of a recurrence from a date, at a time of day
    Expand(Rrule, Date, u8, u8),
    /// UTC to local time in a zone
    ToLocal(u8, i64),
    /// Local time in a zone to UTC
    ToUtc(u8, DateTime),
}

pub struct Schedule {
    events: Vec<Event>,
}

impl Workload for Schedule {
    const NAME: &'static str = "schedule";
    const WEIGHT: u32 = 10;

    fn new(rng: &mut fastrand::Rng) -> Self {
        let events = (0..EVENTS)
            .map(|_| match pick(rng, &[(0, 30), (1, 15), (2, 40), (3, 15)]) {
                0 => Event::CronNext(rng.u8(0..CRONS.len() as u8), near_now(rng, YEAR)),
                1 => {
                    let rule = match rng.u32(0..5) {
                        0 => Rrule::MonthlyNth(pick(rng, &[(1, 30), (2, 30), (3, 10), (-1, 30)]), rng.u8(0..7)),
                        1 => Rrule::MonthlyDay({
                            let c = [(1, 40), (15, 20), (31, 20), (rng.u8(1..=31), 20)];
                            pick(rng, &c)
                        }),
                        2 => Rrule::YearlyNth(rng.u8(1..=12), pick(rng, &[(1, 30), (2, 20), (4, 20), (-1, 30)]), rng.u8(0..7)),
                        3 => Rrule::Weekly({
                            let c = [(0b0010_1010, 40), (0b0011_1110, 30), (1 << rng.u8(1..=7), 30)];
                            pick(rng, &c)
                        }),
                        _ => Rrule::Daily(rng.u8(1..=7)),
                    };
                    let start = datealgo::rd_to_date((near_now(rng, YEAR) / DAY) as i32);
                    Event::Expand(rule, start, rng.u8(9..=17), rng.u8(5..=20))
                }
                2 => Event::ToLocal(rng.u8(0..ZONES.len() as u8), near_now(rng, 2 * YEAR)),
                _ => Event::ToUtc(rng.u8(0..ZONES.len() as u8), datealgo::secs_to_datetime(near_now(rng, 2 * YEAR))),
            })
            .collect();
        Schedule { events }
    }

    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64 {
        let dep_i = dep as i64;
        match self.events[i % EVENTS] {
            Event::CronNext(k, now) => cron_next::<C>(&CRONS[k as usize], now + dep_i),
            Event::Expand(rule, (y, m, d), hour, count) => expand::<C>(rule, (y, m, d + dep as u8), hour, count),
            Event::ToLocal(z, t) => to_local::<C>(&ZONES[z as usize], t + dep_i),
            Event::ToUtc(z, (y, m, d, hh, mm, ss)) => to_utc::<C>(&ZONES[z as usize], (y, m, d, hh, mm + dep as u8, ss)),
        }
    }
}

/// Whether a day matches a cron schedule. As in cron, a schedule restricting
/// both the day of the month and the day of the week matches either.
fn day_matches<C: Calendar>(c: &Cron, (y, m, d): Date) -> bool {
    let dom = c.days >> d & 1 == 1 || (c.last_day && d == C::days_in_month(y, m));
    let dom_all = c.days == ALL_DAYS;
    let wd = C::date_to_weekday((y, m, d)) % 7;
    let dow = match c.nth {
        Some((w, n)) => wd == w && (d - 1) / 7 + 1 == n,
        None => c.weekdays >> wd & 1 == 1,
    };
    let dow_all = c.weekdays == ALL_WEEKDAYS && c.nth.is_none();
    match (dom_all, dow_all) {
        (true, _) => dow,
        (_, true) => dom,
        _ => dom || dow,
    }
}

#[inline(never)]
fn cron_next<C: Calendar>(c: &Cron, now: i64) -> u64 {
    black_box(C::SALT);
    // Search month by month and day by day from the next minute, as the cron
    // crate does, for at most eight years
    let (mut y, mut m, mut d, h0, mi0, _) = C::secs_to_datetime(now + 60);
    let (mut h_from, mut mi_from) = (h0, mi0);
    for _ in 0..96 {
        if c.months >> m & 1 == 1 {
            let dim = C::days_in_month(y, m);
            while d <= dim {
                if day_matches::<C>(c, (y, m, d)) {
                    for h in h_from..24 {
                        if c.hours >> h & 1 == 0 {
                            continue;
                        }
                        let from = if h == h_from { mi_from } else { 0 };
                        if let Some(mi) = (from..60).find(|&mi| c.minutes >> mi & 1 == 1) {
                            return C::datetime_to_secs((y, m, d, h, mi, 0)) as u64;
                        }
                    }
                }
                d += 1;
                (h_from, mi_from) = (0, 0);
            }
        }
        (m, d, h_from, mi_from) = (m % 12 + 1, 1, 0, 0);
        y += (m == 1) as i32;
    }
    0
}

#[inline(never)]
fn expand<C: Calendar>(rule: Rrule, start: Date, hour: u8, count: u8) -> u64 {
    black_box(C::SALT);
    let mut h = 0u64;
    let emit = |h: &mut u64, d: Date| *h = text::mix(*h, C::datetime_to_secs((d.0, d.1, d.2, hour, 0, 0)) as u64);
    let (mut y, mut m, _) = start;
    let mut n = 0;
    match rule {
        Rrule::MonthlyNth(nth, wd) => {
            while n < count {
                emit(&mut h, nth_weekday::<C>(y, m, nth, wd));
                (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                n += 1;
            }
        }
        Rrule::MonthlyDay(day) => {
            for _ in 0..count as u32 * 2 {
                if n == count {
                    break;
                }
                if day <= C::days_in_month(y, m) {
                    emit(&mut h, (y, m, day));
                    n += 1;
                }
                (y, m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
            }
        }
        Rrule::YearlyNth(month, nth, wd) => {
            while n < count {
                emit(&mut h, nth_weekday::<C>(y, month, nth, wd));
                y += 1;
                n += 1;
            }
        }
        Rrule::Weekly(days) => {
            // Day by day, checking the day of the week, as rrule does with its
            // masks
            let mut d = start;
            let mut wd = C::date_to_weekday(d);
            while n < count {
                if days >> wd & 1 == 1 {
                    emit(&mut h, d);
                    n += 1;
                }
                d = C::next_date(d);
                wd = wd % 7 + 1;
            }
        }
        Rrule::Daily(interval) => {
            let mut rd = C::date_to_rd(start);
            while n < count {
                emit(&mut h, C::rd_to_date(rd));
                rd += interval as i32;
                n += 1;
            }
        }
    }
    h
}

/// The nth given day of the week (Sunday is 0) in a month, with -1 for the
/// last
fn nth_weekday<C: Calendar>(y: i32, m: u8, nth: i8, wd: u8) -> Date {
    if nth < 0 {
        let last = C::date_to_rd((y, m, C::days_in_month(y, m)));
        C::rd_to_date(last - (C::rd_to_weekday(last) as i32 % 7 - wd as i32).rem_euclid(7))
    } else {
        let first = C::date_to_weekday((y, m, 1)) % 7;
        (y, m, 1 + (wd as i32 - first as i32).rem_euclid(7) as u8 + (nth as u8 - 1) * 7)
    }
}

/// The UTC time of a POSIX rule transition in a year, from the offset in
/// effect before it
fn transition<C: Calendar>(y: i32, (m, week, wd, hour): (u8, u8, u8, u8), offset: i32) -> i64 {
    // "Mm.w.d": week 5 means the last such day of the month
    let first = C::date_to_rd((y, m, 1));
    let mut day = first + (wd as i32 - C::rd_to_weekday(first) as i32 % 7).rem_euclid(7) + (week as i32 - 1) * 7;
    if week == 5 && day >= first + C::days_in_month(y, m) as i32 {
        day -= 7;
    }
    C::dhms_to_secs((day, hour, 0, 0)) - offset as i64
}

/// The offset of a zone at a UTC time
fn offset_at<C: Calendar>(z: &Zone, t: i64) -> i32 {
    if z.dst == 0 {
        return z.std;
    }
    let (y, _, _, _, _, _) = C::secs_to_datetime(t + z.std as i64);
    let start = transition::<C>(y, z.start, z.std);
    let end = transition::<C>(y, z.end, z.dst);
    let dst = if start < end {
        t >= start && t < end
    } else {
        t >= start || t < end
    };
    if dst {
        z.dst
    } else {
        z.std
    }
}

#[inline(never)]
fn to_local<C: Calendar>(z: &Zone, t: i64) -> u64 {
    black_box(C::SALT);
    let offset = offset_at::<C>(z, t);
    let (y, m, d, hh, mm, ss) = C::secs_to_datetime(t + offset as i64);
    (y as u64) << 40 | (m as u64) << 32 | (d as u64) << 24 | (hh as u64) << 16 | (mm as u64) << 8 | ss as u64 ^ (offset as u64) << 48
}

#[inline(never)]
fn to_utc<C: Calendar>(z: &Zone, local: DateTime) -> u64 {
    black_box(C::SALT);
    // Try the standard offset, and the DST offset if that is not in effect,
    // picking the earlier time when the local time is ambiguous
    let secs = C::datetime_to_secs(local);
    let t = secs - z.std as i64;
    if offset_at::<C>(z, t) == z.std {
        t as u64
    } else {
        (secs - z.dst as i64) as u64
    }
}
