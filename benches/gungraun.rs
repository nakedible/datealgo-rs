// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use gungraun::prelude::*;
use std::{
    hint::black_box,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[library_benchmark]
fn rd_to_date() -> (i32, u8, u8) {
    datealgo::rd_to_date(black_box(19489))
}

#[library_benchmark]
fn date_to_rd() -> i32 {
    datealgo::date_to_rd(black_box((2023, 5, 12)))
}

#[library_benchmark]
fn rd_to_weekday() -> u8 {
    datealgo::rd_to_weekday(black_box(19489))
}

#[library_benchmark]
fn date_to_weekday() -> u8 {
    datealgo::date_to_weekday(black_box((2023, 5, 12)))
}

#[library_benchmark]
#[bench::mid_month((2023, 5, 12))]
#[bench::month_end((2023, 5, 31))]
#[bench::year_end((2023, 12, 31))]
fn next_date(date: (i32, u8, u8)) -> (i32, u8, u8) {
    datealgo::next_date(black_box(date))
}

#[library_benchmark]
#[bench::mid_month((2023, 5, 12))]
#[bench::month_start((2023, 5, 1))]
#[bench::year_start((2023, 1, 1))]
fn prev_date(date: (i32, u8, u8)) -> (i32, u8, u8) {
    datealgo::prev_date(black_box(date))
}

#[library_benchmark]
fn secs_to_dhms() -> (i32, u8, u8, u8) {
    datealgo::secs_to_dhms(black_box(1684574678))
}

#[library_benchmark]
fn dhms_to_secs() -> i64 {
    datealgo::dhms_to_secs(black_box((19489, 12, 34, 56)))
}

#[library_benchmark]
fn secs_to_datetime() -> (i32, u8, u8, u8, u8, u8) {
    datealgo::secs_to_datetime(black_box(1684574678))
}

#[library_benchmark]
fn datetime_to_secs() -> i64 {
    datealgo::datetime_to_secs(black_box((2023, 5, 20, 12, 34, 56)))
}

#[library_benchmark]
fn is_leap_year() -> bool {
    datealgo::is_leap_year(black_box(2000))
}

#[library_benchmark]
fn days_in_month() -> u8 {
    datealgo::days_in_month(black_box(2000), black_box(2))
}

#[library_benchmark]
fn rd_to_isoweekdate() -> (i32, u8, u8) {
    datealgo::rd_to_isoweekdate(black_box(19489))
}

#[library_benchmark]
fn isoweekdate_to_rd() -> i32 {
    datealgo::isoweekdate_to_rd(black_box((2023, 19, 5)))
}

#[library_benchmark]
fn date_to_isoweekdate() -> (i32, u8, u8) {
    datealgo::date_to_isoweekdate(black_box((2023, 5, 12)))
}

#[library_benchmark]
fn isoweekdate_to_date() -> (i32, u8, u8) {
    datealgo::isoweekdate_to_date(black_box((2023, 19, 5)))
}

#[library_benchmark]
fn isoweeks_in_year() -> u8 {
    datealgo::isoweeks_in_year(black_box(2023))
}

#[library_benchmark]
#[bench::after_epoch(UNIX_EPOCH + Duration::from_secs(1684574678))]
#[bench::before_epoch(UNIX_EPOCH - Duration::new(1684574678, 500_000_000))]
fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
    datealgo::systemtime_to_secs(black_box(st))
}

#[library_benchmark]
#[bench::after_epoch((1684574678, 0))]
#[bench::before_epoch((-1684574678, 500_000_000))]
fn secs_to_systemtime(secs: (i64, u32)) -> Option<SystemTime> {
    datealgo::secs_to_systemtime(black_box(secs))
}

#[library_benchmark]
fn systemtime_to_datetime() -> Option<(i32, u8, u8, u8, u8, u8, u32)> {
    datealgo::systemtime_to_datetime(black_box(UNIX_EPOCH + Duration::from_secs(1684574678)))
}

#[library_benchmark]
fn datetime_to_systemtime() -> Option<SystemTime> {
    datealgo::datetime_to_systemtime(black_box((2023, 5, 20, 12, 34, 56, 0)))
}

library_benchmark_group!(
    name = datealgo_group,
    benchmarks = [
        rd_to_date,
        date_to_rd,
        rd_to_weekday,
        date_to_weekday,
        next_date,
        prev_date,
        secs_to_dhms,
        dhms_to_secs,
        secs_to_datetime,
        datetime_to_secs,
        is_leap_year,
        days_in_month,
        rd_to_isoweekdate,
        isoweekdate_to_rd,
        date_to_isoweekdate,
        isoweekdate_to_date,
        isoweeks_in_year,
        systemtime_to_secs,
        secs_to_systemtime,
        systemtime_to_datetime,
        datetime_to_systemtime,
    ]
);

main!(library_benchmark_groups = [datealgo_group]);
