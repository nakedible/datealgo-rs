use iai_callgrind::{black_box, main};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[inline(never)]
fn iai_rd_to_date() -> (i32, u32, u32) {
    datealgo::rd_to_date(black_box(12345))
}

#[inline(never)]
fn iai_date_to_rd() -> i32 {
    datealgo::date_to_rd(black_box((2023, 5, 12)))
}

#[inline(never)]
fn iai_rd_to_weekday() -> u32 {
    datealgo::rd_to_weekday(black_box(12345))
}

#[inline(never)]
fn iai_date_to_weekday() -> u32 {
    datealgo::date_to_weekday(black_box((2023, 5, 12)))
}

#[inline(never)]
fn iai_secs_to_dhms() -> (i32, u8, u8, u8) {
    datealgo::secs_to_dhms(black_box(1684574678i64))
}

#[inline(never)]
fn iai_dhms_to_secs() -> i64 {
    datealgo::dhms_to_secs(black_box((123123, 12, 34, 56)))
}

#[inline(never)]
fn iai_secs_to_datetime() -> (i32, u32, u32, u8, u8, u8) {
    datealgo::secs_to_datetime(black_box(1684574678i64))
}

#[inline(never)]
fn iai_datetime_to_secs() -> i64 {
    datealgo::datetime_to_secs(black_box((2023, 5, 20, 12, 34, 56)))
}

#[inline(never)]
fn iai_is_leap_year() -> bool {
    datealgo::is_leap_year(black_box(2000))
}

#[inline(never)]
fn iai_days_in_month() -> u32 {
    datealgo::days_in_month(black_box(2000), black_box(2))
}

#[inline(never)]
fn iai_systemtime_to_secs() -> Option<(i64, u32)> {
    datealgo::systemtime_to_secs(black_box(UNIX_EPOCH + Duration::from_secs(1684574678)))
}

#[inline(never)]
fn iai_secs_to_systemtime() -> SystemTime {
    datealgo::secs_to_systemtime(black_box((1684574678, 0)))
}

#[inline(never)]
fn iai_systemtime_to_datetime() -> Option<(i32, u32, u32, u8, u8, u8, u32)> {
    datealgo::systemtime_to_datetime(black_box(UNIX_EPOCH + Duration::from_secs(1684574678)))
}

#[inline(never)]
fn iai_datetime_to_systemtime() -> SystemTime {
    datealgo::datetime_to_systemtime(black_box((2023, 5, 20, 12, 34, 56, 0)))
}

main!(
    iai_rd_to_date,
    iai_date_to_rd,
    iai_rd_to_weekday,
    iai_date_to_weekday,
    iai_secs_to_dhms,
    iai_dhms_to_secs,
    iai_secs_to_datetime,
    iai_datetime_to_secs,
    iai_is_leap_year,
    iai_days_in_month,
    iai_systemtime_to_secs,
    iai_secs_to_systemtime,
    iai_systemtime_to_datetime,
    iai_datetime_to_systemtime,
);
