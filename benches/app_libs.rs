//! Application benchmark comparing datealgo with other libraries in each
//! domain of the synthetic application of `benches/app/`. Run with
//! `scripts/app_libs.sh`.
//!
//! Each library implements the calendar functions it has with its own types
//! and operations, as its users would call them, and the others are built from
//! those, as its users would write them. The text parsing and formatting
//! around the calendar code is the same for all rows, so the rows compare only
//! the calendar parts of each library.

// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use std::time::Duration;
use zenbench::prelude::*;

#[macro_use]
#[path = "app/mod.rs"]
mod app;
use app::*;

mod libs {
    //! Adapters for other libraries
    use super::app::{Calendar, Date, DateTime, DateTimeNanos, IsoWeekDate};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Days from 0001-01-01, counted from 1, to 1970-01-01
    const RD_UNIX_EPOCH: i32 = 719_163;
    /// Julian day of 1970-01-01
    const JD_UNIX_EPOCH: i32 = 2_440_588;

    pub struct Chrono;
    impl Chrono {
        fn date((y, m, d): Date) -> chrono::NaiveDate {
            chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32).unwrap()
        }
        fn fields(d: chrono::NaiveDate) -> Date {
            use chrono::Datelike;
            (d.year(), d.month() as u8, d.day() as u8)
        }
    }
    impl Calendar for Chrono {
        fn rd_to_date(n: i32) -> Date {
            Self::fields(chrono::NaiveDate::from_num_days_from_ce_opt(n + RD_UNIX_EPOCH).unwrap())
        }
        fn date_to_rd(d: Date) -> i32 {
            use chrono::Datelike;
            Self::date(d).num_days_from_ce() - RD_UNIX_EPOCH
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            // chrono has no days in month, so users take the day before the
            // first of the next month
            use chrono::Datelike;
            let next = if m == 12 { (y + 1, 1) } else { (y, m as u32 + 1) };
            chrono::NaiveDate::from_ymd_opt(next.0, next.1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
                .day() as u8
        }
        fn rd_to_weekday(n: i32) -> u8 {
            use chrono::Datelike;
            chrono::NaiveDate::from_num_days_from_ce_opt(n + RD_UNIX_EPOCH)
                .unwrap()
                .weekday()
                .number_from_monday() as u8
        }
        fn date_to_weekday(d: Date) -> u8 {
            use chrono::Datelike;
            Self::date(d).weekday().number_from_monday() as u8
        }
        fn next_date(d: Date) -> Date {
            Self::fields(Self::date(d).succ_opt().unwrap())
        }
        fn prev_date(d: Date) -> Date {
            Self::fields(Self::date(d).pred_opt().unwrap())
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
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
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            Self::date((y, m, d))
                .and_hms_opt(hh as u32, mm as u32, ss as u32)
                .unwrap()
                .and_utc()
                .timestamp()
        }
        fn is_leap_year(y: i32) -> bool {
            chrono::NaiveDate::from_ymd_opt(y, 1, 1).unwrap().leap_year()
        }
        fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
            use chrono::Datelike;
            let date = Self::date(d);
            let w = date.iso_week();
            (w.year(), w.week() as u8, date.weekday().number_from_monday() as u8)
        }
        fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
            Self::date_to_isoweekdate(Self::rd_to_date(n))
        }
        fn isoweekdate_to_date((y, w, d): IsoWeekDate) -> Date {
            let wd = chrono::Weekday::try_from(d - 1).unwrap();
            Self::fields(chrono::NaiveDate::from_isoywd_opt(y, w as u32, wd).unwrap())
        }
        fn isoweekdate_to_rd(iwd: IsoWeekDate) -> i32 {
            Self::date_to_rd(Self::isoweekdate_to_date(iwd))
        }
        fn isoweeks_in_year(y: i32) -> u8 {
            use chrono::Datelike;
            chrono::NaiveDate::from_ymd_opt(y, 12, 28).unwrap().iso_week().week() as u8
        }
        fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
            use chrono::{Datelike, Timelike};
            let t: chrono::DateTime<chrono::Utc> = st.into();
            Some((
                t.year(),
                t.month() as u8,
                t.day() as u8,
                t.hour() as u8,
                t.minute() as u8,
                t.second() as u8,
                t.nanosecond(),
            ))
        }
        fn datetime_to_systemtime((y, m, d, hh, mm, ss, ns): DateTimeNanos) -> Option<SystemTime> {
            Some(
                chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32)?
                    .and_hms_nano_opt(hh as u32, mm as u32, ss as u32, ns)?
                    .and_utc()
                    .into(),
            )
        }
    }

    pub struct Time;
    impl Time {
        fn date((y, m, d): Date) -> time::Date {
            time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
        }
        fn fields(d: time::Date) -> Date {
            let (y, m, d) = d.to_calendar_date();
            (y, m as u8, d)
        }
    }
    impl Calendar for Time {
        fn rd_to_date(n: i32) -> Date {
            Self::fields(time::Date::from_julian_day(n + JD_UNIX_EPOCH).unwrap())
        }
        fn date_to_rd(d: Date) -> i32 {
            Self::date(d).to_julian_day() - JD_UNIX_EPOCH
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            time::Month::try_from(m).unwrap().length(y)
        }
        fn rd_to_weekday(n: i32) -> u8 {
            time::Date::from_julian_day(n + JD_UNIX_EPOCH)
                .unwrap()
                .weekday()
                .number_from_monday()
        }
        fn date_to_weekday(d: Date) -> u8 {
            Self::date(d).weekday().number_from_monday()
        }
        fn next_date(d: Date) -> Date {
            Self::fields(Self::date(d).next_day().unwrap())
        }
        fn prev_date(d: Date) -> Date {
            Self::fields(Self::date(d).previous_day().unwrap())
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
            let t = time::UtcDateTime::from_unix_timestamp(secs).unwrap();
            let (y, m, d) = t.to_calendar_date();
            let (hh, mm, ss) = t.as_hms();
            (y, m as u8, d, hh, mm, ss)
        }
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            Self::date((y, m, d)).with_hms(hh, mm, ss).unwrap().as_utc().unix_timestamp()
        }
        fn is_leap_year(y: i32) -> bool {
            time::util::is_leap_year(y)
        }
        fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
            let (y, w, wd) = Self::date(d).to_iso_week_date();
            (y, w, wd.number_from_monday())
        }
        fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
            let (y, w, wd) = time::Date::from_julian_day(n + JD_UNIX_EPOCH).unwrap().to_iso_week_date();
            (y, w, wd.number_from_monday())
        }
        fn isoweekdate_to_date((y, w, d): IsoWeekDate) -> Date {
            Self::fields(time::Date::from_iso_week_date(y, w, time::Weekday::Sunday.nth_next(d)).unwrap())
        }
        fn isoweekdate_to_rd(iwd: IsoWeekDate) -> i32 {
            Self::date_to_rd(Self::isoweekdate_to_date(iwd))
        }
        fn isoweeks_in_year(y: i32) -> u8 {
            time::util::weeks_in_year(y)
        }
        fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
            let t: time::UtcDateTime = st.into();
            let (y, m, d) = t.to_calendar_date();
            let (hh, mm, ss, ns) = t.as_hms_nano();
            Some((y, m as u8, d, hh, mm, ss, ns))
        }
        fn datetime_to_systemtime((y, m, d, hh, mm, ss, ns): DateTimeNanos) -> Option<SystemTime> {
            let date = time::Date::from_calendar_date(y, time::Month::try_from(m).ok()?, d).ok()?;
            Some(date.with_hms_nano(hh, mm, ss, ns).ok()?.as_utc().into())
        }
    }

    /// ICU4X `Date<Iso>`, which has no timestamps, so those are built from
    /// its dates
    pub struct Icu;
    impl Icu {
        fn from_rd(n: i32) -> icu_calendar::Date<icu_calendar::Iso> {
            icu_calendar::Date::from_rata_die(icu_calendar::types::RataDie::new((n + RD_UNIX_EPOCH) as i64), icu_calendar::Iso)
        }
        fn fields(d: &icu_calendar::Date<icu_calendar::Iso>) -> Date {
            (d.year().extended_year(), d.month().ordinal, d.day_of_month().0)
        }
        fn date((y, m, d): Date) -> icu_calendar::Date<icu_calendar::Iso> {
            icu_calendar::Date::try_new_iso(y, m, d).unwrap()
        }
    }
    impl Calendar for Icu {
        fn rd_to_date(n: i32) -> Date {
            Self::fields(&Self::from_rd(n))
        }
        fn date_to_rd(d: Date) -> i32 {
            (Self::date(d).to_rata_die().to_i64_date() - RD_UNIX_EPOCH as i64) as i32
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            Self::date((y, m, 1)).days_in_month()
        }
        fn rd_to_weekday(n: i32) -> u8 {
            Self::from_rd(n).weekday() as u8
        }
        fn date_to_weekday(d: Date) -> u8 {
            Self::date(d).weekday() as u8
        }
        fn is_leap_year(y: i32) -> bool {
            Self::date((y, 1, 1)).is_in_leap_year()
        }
        fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
            let date = Self::date(d);
            let w = date.week_of_year();
            (w.iso_year, w.week_number, date.weekday() as u8)
        }
        fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
            let date = Self::from_rd(n);
            let w = date.week_of_year();
            (w.iso_year, w.week_number, date.weekday() as u8)
        }
    }

    pub struct Fasttime;
    impl Calendar for Fasttime {
        fn rd_to_date(n: i32) -> Date {
            let d = fasttime::Date::from_days_since_unix_epoch(n as i64).unwrap();
            (d.year, d.month, d.day)
        }
        fn date_to_rd((y, m, d): Date) -> i32 {
            fasttime::Date::from_ymd(y, m, d).unwrap().days_since_unix_epoch() as i32
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            // fasttime has no days in month, so users find the last valid day
            (28..=31).rev().find(|&d| fasttime::Date::from_ymd(y, m, d).is_ok()).unwrap()
        }
        fn rd_to_weekday(n: i32) -> u8 {
            fasttime::Date::from_days_since_unix_epoch(n as i64)
                .unwrap()
                .weekday()
                .number_from_monday()
        }
        fn date_to_weekday((y, m, d): Date) -> u8 {
            fasttime::Date::from_ymd(y, m, d).unwrap().weekday().number_from_monday()
        }
        fn next_date((y, m, d): Date) -> Date {
            let d = fasttime::Date::from_ymd(y, m, d).unwrap().add_days(1).unwrap();
            (d.year, d.month, d.day)
        }
        fn prev_date((y, m, d): Date) -> Date {
            let d = fasttime::Date::from_ymd(y, m, d).unwrap().add_days(-1).unwrap();
            (d.year, d.month, d.day)
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
            let t = fasttime::DateTime::from_unix_timestamp(secs, 0).unwrap();
            (t.date.year, t.date.month, t.date.day, t.time.hour, t.time.minute, t.time.second)
        }
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            let date = fasttime::Date::from_ymd(y, m, d).unwrap();
            let time = fasttime::Time {
                hour: hh,
                minute: mm,
                second: ss,
                nanosecond: 0,
            };
            fasttime::DateTime::new(date, time).unix_timestamp()
        }
    }

    /// hifitime, which works in TAI and applies leap seconds
    pub struct Hifitime;
    impl Hifitime {
        fn midnight(n: i32) -> hifitime::Epoch {
            hifitime::Epoch::from_unix_duration(hifitime::Duration::from_total_nanoseconds(n as i128 * 86_400_000_000_000))
        }
    }
    impl Calendar for Hifitime {
        fn rd_to_date(n: i32) -> Date {
            let (y, m, d, ..) = Self::midnight(n).to_gregorian_utc();
            (y, m, d)
        }
        fn date_to_rd((y, m, d): Date) -> i32 {
            let e = hifitime::Epoch::from_gregorian_utc_at_midnight(y, m, d);
            (e.to_unix_duration().total_nanoseconds().div_euclid(86_400_000_000_000)) as i32
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            (28..=31)
                .rev()
                .find(|&d| hifitime::Epoch::maybe_from_gregorian_utc(y, m, d, 0, 0, 0, 0).is_ok())
                .unwrap()
        }
        fn rd_to_weekday(n: i32) -> u8 {
            Self::midnight(n).weekday_utc() as u8 + 1
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
            let e = hifitime::Epoch::from_unix_duration(hifitime::Duration::from_total_nanoseconds(secs as i128 * 1_000_000_000));
            let (y, m, d, hh, mm, ss, _) = e.to_gregorian_utc();
            (y, m, d, hh, mm, ss)
        }
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            let e = hifitime::Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0);
            (e.to_unix_duration().total_nanoseconds().div_euclid(1_000_000_000)) as i64
        }
    }

    /// jiff-core, the low-level date primitives of jiff, with i16 years and
    /// validating constructors. Its timestamps end before the last day of 9999,
    /// so seconds go through days and the second of the day, as jiff does.
    pub struct JiffCore;
    impl JiffCore {
        fn date((y, m, d): Date) -> jiff_core::civil::Date {
            jiff_core::civil::Date::new(y as i16, m as i8, d as i8).unwrap()
        }
        fn fields(d: jiff_core::civil::Date) -> Date {
            (d.year() as i32, d.month() as u8, d.day() as u8)
        }
        fn iso(w: jiff_core::civil::ISOWeekDate) -> IsoWeekDate {
            (w.year() as i32, w.week() as u8, w.weekday().to_monday_one_offset() as u8)
        }
        fn weekday(d: u8) -> jiff_core::civil::Weekday {
            jiff_core::civil::Weekday::from_monday_one_offset(d as i8).unwrap()
        }
    }
    impl Calendar for JiffCore {
        fn rd_to_date(n: i32) -> Date {
            Self::fields(jiff_core::civil::UnixEpochDay::new(n).unwrap().to_date())
        }
        fn date_to_rd(d: Date) -> i32 {
            Self::date(d).to_unix_epoch_day().day()
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            Self::date((y, m, 1)).days_in_month() as u8
        }
        fn rd_to_weekday(n: i32) -> u8 {
            jiff_core::civil::UnixEpochDay::new(n).unwrap().weekday().to_monday_one_offset() as u8
        }
        fn date_to_weekday(d: Date) -> u8 {
            Self::date(d).weekday().to_monday_one_offset() as u8
        }
        fn next_date(d: Date) -> Date {
            Self::fields(Self::date(d).tomorrow().unwrap())
        }
        fn prev_date(d: Date) -> Date {
            Self::fields(Self::date(d).yesterday().unwrap())
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
            let d = jiff_core::civil::UnixEpochDay::new(secs.div_euclid(86400) as i32)
                .unwrap()
                .to_date();
            let t = jiff_core::civil::TimeSecond::new(secs.rem_euclid(86400) as i32).unwrap().to_time();
            (
                d.year() as i32,
                d.month() as u8,
                d.day() as u8,
                t.hour() as u8,
                t.minute() as u8,
                t.second() as u8,
            )
        }
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            let t = jiff_core::civil::Time::new(hh as i8, mm as i8, ss as i8, 0).unwrap();
            Self::date((y, m, d)).to_unix_epoch_day().day() as i64 * 86400 + t.to_second().second() as i64
        }
        fn is_leap_year(y: i32) -> bool {
            Self::date((y, 1, 1)).in_leap_year()
        }
        fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
            Self::iso(Self::date(d).to_iso_week_date())
        }
        fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
            Self::iso(jiff_core::civil::UnixEpochDay::new(n).unwrap().to_date().to_iso_week_date())
        }
        fn isoweekdate_to_date((y, w, d): IsoWeekDate) -> Date {
            Self::fields(
                jiff_core::civil::ISOWeekDate::new(y as i16, w as i8, Self::weekday(d))
                    .unwrap()
                    .to_date(),
            )
        }
        fn isoweekdate_to_rd((y, w, d): IsoWeekDate) -> i32 {
            jiff_core::civil::ISOWeekDate::new(y as i16, w as i8, Self::weekday(d))
                .unwrap()
                .to_unix_epoch_day()
                .day()
        }
        fn isoweeks_in_year(y: i32) -> u8 {
            jiff_core::civil::ISOWeekDate::new(y as i16, 1, Self::weekday(1))
                .unwrap()
                .weeks_in_year() as u8
        }
    }

    /// jiff, used as its users would. Its timestamps end before the last day
    /// of 9999 and it has no days since the epoch, so this counts from the
    /// epoch with civil dates and datetimes.
    pub struct Jiff;
    impl Jiff {
        const EPOCH: jiff::civil::DateTime = jiff::civil::DateTime::constant(1970, 1, 1, 0, 0, 0, 0);
        fn date((y, m, d): Date) -> jiff::civil::Date {
            jiff::civil::Date::new(y as i16, m as i8, d as i8).unwrap()
        }
        fn fields(d: jiff::civil::Date) -> Date {
            (d.year() as i32, d.month() as u8, d.day() as u8)
        }
        fn from_rd(n: i32) -> jiff::civil::Date {
            Self::EPOCH.date().checked_add(jiff::Span::new().days(n)).unwrap()
        }
        fn iso(w: jiff::civil::ISOWeekDate) -> IsoWeekDate {
            (w.year() as i32, w.week() as u8, w.weekday().to_monday_one_offset() as u8)
        }
    }
    impl Calendar for Jiff {
        fn rd_to_date(n: i32) -> Date {
            Self::fields(Self::from_rd(n))
        }
        fn date_to_rd(d: Date) -> i32 {
            (Self::date(d).duration_since(Self::EPOCH.date()).as_secs() / 86400) as i32
        }
        fn days_in_month(y: i32, m: u8) -> u8 {
            Self::date((y, m, 1)).days_in_month() as u8
        }
        fn rd_to_weekday(n: i32) -> u8 {
            Self::from_rd(n).weekday().to_monday_one_offset() as u8
        }
        fn date_to_weekday(d: Date) -> u8 {
            Self::date(d).weekday().to_monday_one_offset() as u8
        }
        fn next_date(d: Date) -> Date {
            Self::fields(Self::date(d).tomorrow().unwrap())
        }
        fn prev_date(d: Date) -> Date {
            Self::fields(Self::date(d).yesterday().unwrap())
        }
        fn secs_to_datetime(secs: i64) -> DateTime {
            let dt = Self::EPOCH.checked_add(jiff::SignedDuration::from_secs(secs)).unwrap();
            (
                dt.year() as i32,
                dt.month() as u8,
                dt.day() as u8,
                dt.hour() as u8,
                dt.minute() as u8,
                dt.second() as u8,
            )
        }
        fn datetime_to_secs((y, m, d, hh, mm, ss): DateTime) -> i64 {
            let dt = jiff::civil::DateTime::new(y as i16, m as i8, d as i8, hh as i8, mm as i8, ss as i8, 0).unwrap();
            dt.duration_since(Self::EPOCH).as_secs()
        }
        fn is_leap_year(y: i32) -> bool {
            Self::date((y, 1, 1)).in_leap_year()
        }
        fn date_to_isoweekdate(d: Date) -> IsoWeekDate {
            Self::iso(Self::date(d).iso_week_date())
        }
        fn rd_to_isoweekdate(n: i32) -> IsoWeekDate {
            Self::iso(Self::from_rd(n).iso_week_date())
        }
        fn isoweekdate_to_date((y, w, d): IsoWeekDate) -> Date {
            let wd = jiff::civil::Weekday::from_monday_one_offset(d as i8).unwrap();
            Self::fields(jiff::civil::ISOWeekDate::new(y as i16, w as i8, wd).unwrap().date())
        }
        fn systemtime_to_datetime(st: SystemTime) -> Option<DateTimeNanos> {
            let since = jiff::SignedDuration::system_until(UNIX_EPOCH, st).ok()?;
            let dt = Self::EPOCH.checked_add(since).ok()?;
            Some((
                dt.year() as i32,
                dt.month() as u8,
                dt.day() as u8,
                dt.hour() as u8,
                dt.minute() as u8,
                dt.second() as u8,
                dt.subsec_nanosecond() as u32,
            ))
        }
        fn datetime_to_systemtime((y, m, d, hh, mm, ss, ns): DateTimeNanos) -> Option<SystemTime> {
            let dt = jiff::civil::DateTime::new(y as i16, m as i8, d as i8, hh as i8, mm as i8, ss as i8, ns as i32).ok()?;
            let since = dt.duration_since(Self::EPOCH);
            if since.is_negative() {
                UNIX_EPOCH.checked_sub(since.unsigned_abs())
            } else {
                UNIX_EPOCH.checked_add(since.unsigned_abs())
            }
        }
    }

    /// temporal_rs, which only provides the date conversions, so the rest is
    /// built from them
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

fn bench<C: Calendar + 'static, W: Workload>(group: &mut BenchGroup, name: &str) {
    group.bench(name.to_owned(), move |b| {
        let (w, rest) = (workload::<W>(), Rest::get());
        let mut start = 0;
        let mut acc = 0;
        b.iter(move || {
            acc = run_placed::<C, W>(w, rest, start, acc);
            start = (start + CHUNK) % EVENTS;
            acc
        })
    });
}

fn configure(group: &mut BenchGroup) {
    // Seconds to measure each domain for, 30 by default
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

/// Register a group for a domain with a row for each library, after checking
/// that the results of the trace match those of datealgo. A mismatch is
/// reported, as results may differ for reasons like leap seconds.
fn domain<W: Workload>(suite: &mut Suite) {
    let expected = checksum::<Datealgo, W>();
    suite.group(W::NAME, |group| {
        configure(group);
        macro_rules! row {
            ($name:expr, $cal:ty) => {
                if checksum::<$cal, W>() != expected {
                    eprintln!("{} {}: results differ from datealgo", W::NAME, $name);
                }
                bench::<$cal, W>(group, $name);
            };
        }
        bench::<Datealgo, W>(group, "datealgo");
        row!("chrono", libs::Chrono);
        row!("time", libs::Time);
        row!("jiff", libs::Jiff);
        row!("jiff_core", libs::JiffCore);
        row!("icu", libs::Icu);
        row!("fasttime", libs::Fasttime);
        row!("hifitime", libs::Hifitime);
        row!("temporal", libs::Temporal);
        bench::<NoCalendar, W>(group, "none");
    });
}

fn app(suite: &mut Suite) {
    macro_rules! register {
        ($m:ident, $t:ty) => {
            domain::<$t>(suite);
        };
    }
    for_each_domain!(register);
}

zenbench::main!(app);
