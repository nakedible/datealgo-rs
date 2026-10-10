//! Application benchmark comparing datealgo with other libraries in the
//! synthetic application of `app_support.rs`. Run with `scripts/app_libs.sh`.
//!
//! The string parsing and formatting around the calendar code is the same for
//! all rows, so the rows compare only the calendar parts of each library.

// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

#[macro_use]
#[path = "app_support.rs"]
mod support;
use support::*;

mod libs {
    //! Adapters for other libraries, using each library's own types and
    //! operations as its users would.
    use super::support::{Calendar, Date, DateTime};
    use timezone_provider::utils::{
        iso_days_in_month,
        neri_schneider::{epoch_days_from_gregorian_date, ymd_from_epoch_days},
    };

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
            (
                t.year(),
                t.month() as u8,
                t.day() as u8,
                t.hour() as u8,
                t.minute() as u8,
                t.second() as u8,
            )
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

    /// temporal_rs, which only provides the date conversions, so the rest
    /// is datealgo
    pub struct Temporal;
    impl Calendar for Temporal {
        fn to_datetime(secs: i64) -> DateTime {
            let (n, hh, mm, ss) = datealgo::secs_to_dhms(secs);
            let (y, m, d) = ymd_from_epoch_days(n);
            (y, m, d, hh, mm, ss)
        }

        fn from_datetime((y, m, d, hh, mm, ss): DateTime) -> Option<i64> {
            if !(1..=12).contains(&m) || d < 1 || d > iso_days_in_month(y, m) || hh > 23 || mm > 59 || ss > 59 {
                return None;
            }
            let n = epoch_days_from_gregorian_date(y, m, d) as i32;
            Some(datealgo::dhms_to_secs((n, hh, mm, ss)))
        }

        fn shift(secs: i64, days: i32) -> (Date, u8) {
            let n = datealgo::secs_to_dhms(secs).0 + days;
            (ymd_from_epoch_days(n), datealgo::rd_to_weekday(n))
        }

        fn week(secs: i64, out: &mut [Date; 7]) -> u8 {
            let n = datealgo::secs_to_dhms(secs).0;
            for (i, slot) in out.iter_mut().enumerate() {
                *slot = ymd_from_epoch_days(n + i as i32);
            }
            datealgo::rd_to_weekday(n)
        }
    }
}

fn bench<C: Calendar + 'static>(group: &mut BenchGroup, name: &str, app: &Arc<App>) {
    let app = Arc::clone(app);
    group.bench(name.to_owned(), move |b| {
        let app = Arc::clone(&app);
        let mut start = 0;
        let mut acc = 0;
        b.iter(move || {
            acc = app.run_placed::<C>(start, acc);
            start = (start + CHUNK) % EVENTS;
            acc
        })
    });
}

/// Register a row for each implementation, after checking that the results of
/// the trace match those of the first. A mismatch is reported, as results may
/// differ for reasons like leap seconds, or a change that fixes a bug.
macro_rules! rows {
    ($group:expr, $app:expr, [$(($name:expr, $cal:ty)),+ $(,)?]) => {{
        let mut first = None;
        $(
            let sum = $app.checksum::<$cal>();
            if sum != *first.get_or_insert(sum) {
                eprintln!("{}: results differ from the first row", $name);
            }
            bench::<$cal>($group, $name, &$app);
        )+
        bench::<NoCalendar>($group, "none", &$app);
    }};
}

fn configure(group: &mut BenchGroup) {
    // Seconds to measure for, 30 by default
    let secs = std::env::var("APP_BENCH_SECS").map_or(30, |s| s.parse().unwrap());
    group
        .config()
        .auto_rounds(false)
        .max_rounds(100_000)
        .max_time(Duration::from_secs(secs))
        .max_wall_time(Duration::from_secs(2 * secs))
        .baseline_only(true);
    group.throughput(Throughput::Elements(CHUNK as u64)).throughput_unit("events");
}

fn app(suite: &mut Suite) {
    let app = Arc::new(App::new());
    suite.group("libraries", |group| {
        configure(group);
        rows!(
            group,
            app,
            [
                ("datealgo", Datealgo),
                ("chrono", libs::Chrono),
                ("time", libs::Time),
                ("icu", libs::Icu),
                ("fasttime", libs::Fasttime),
                ("hifitime", libs::Hifitime),
                ("temporal", libs::Temporal),
            ]
        );
    });
}

zenbench::main!(app);
