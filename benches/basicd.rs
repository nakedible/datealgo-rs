use std::time::SystemTime;

fn rand_year() -> i32 {
    fastrand::i32(datealgo::YEAR_MIN..=datealgo::YEAR_MAX)
}

fn rand_rd() -> i32 {
    fastrand::i32(datealgo::RD_MIN..=datealgo::RD_MAX)
}

fn rand_date() -> (i32, u8, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    let d = fastrand::u8(1..=datealgo::days_in_month(y, m));
    (y, m, d)
}

fn rand_secs() -> i64 {
    fastrand::i64(datealgo::RD_SECONDS_MIN..=datealgo::RD_SECONDS_MAX)
}

fn rand_hms() -> (u8, u8, u8) {
    let h = fastrand::u8(0..=23);
    let m = fastrand::u8(0..=59);
    let s = fastrand::u8(0..59);
    (h, m, s)
}

fn rand_dhms() -> (i32, u8, u8, u8) {
    let rd = rand_rd();
    let (h, m, s) = rand_hms();
    (rd, h, m, s)
}

fn rand_dt() -> (i32, u8, u8, u8, u8, u8) {
    let (y, m, d) = rand_date();
    let (hh, mm, ss) = rand_hms();
    (y, m, d, hh, mm, ss)
}

fn rand_ym() -> (i32, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    (y, m)
}

fn rand_sn() -> (i64, u32) {
    let s = rand_secs();
    let n = fastrand::u32(0..=999_999_999);
    (s, n)
}

fn rand_st() -> SystemTime {
    datealgo::secs_to_systemtime(rand_sn()).unwrap()
}

fn rand_dtn() -> (i32, u8, u8, u8, u8, u8, u32) {
    let (y, m, d, hh, mm, ss) = rand_dt();
    let n = fastrand::u32(0..=999_999_999);
    (y, m, d, hh, mm, ss, n)
}

fn rand_iwd() -> (i32, u8, u8) {
    datealgo::rd_to_isoweekdate(rand_rd())
}

#[divan::bench]
fn bench_overhead(b: divan::Bencher) {
    b.with_inputs(rand_rd).bench_refs(|i| *divan::black_box(i));
}

#[divan::bench]
fn bench_rd_to_date(b: divan::Bencher) {
    b.with_inputs(rand_rd).bench_refs(|i| datealgo::rd_to_date(divan::black_box(*i)));
}

#[divan::bench]
fn bench_date_to_rd(b: divan::Bencher) {
    b.with_inputs(rand_date).bench_refs(|i| datealgo::date_to_rd(divan::black_box(*i)));
}

#[divan::bench]
fn bench_rd_to_weekday(b: divan::Bencher) {
    b.with_inputs(rand_rd).bench_refs(|i| datealgo::rd_to_weekday(divan::black_box(*i)));
}

#[divan::bench]
fn bench_date_to_weekday(b: divan::Bencher) {
    b.with_inputs(rand_date).bench_refs(|i| datealgo::date_to_weekday(divan::black_box(*i)));
}

#[divan::bench]
fn bench_next_date(b: divan::Bencher) {
    b.with_inputs(rand_date).bench_refs(|i| datealgo::next_date(divan::black_box(*i)));
}

#[divan::bench]
fn bench_prev_date(b: divan::Bencher) {
    b.with_inputs(rand_date).bench_refs(|i| datealgo::prev_date(divan::black_box(*i)));
}

#[divan::bench]
fn bench_secs_to_dhms(b: divan::Bencher) {
    b.with_inputs(rand_secs).bench_refs(|i| datealgo::secs_to_dhms(divan::black_box(*i)));
}

#[divan::bench]
fn bench_dhms_to_secs(b: divan::Bencher) {
    b.with_inputs(rand_dhms).bench_refs(|i| datealgo::dhms_to_secs(divan::black_box(*i)));
}

#[divan::bench]
fn bench_secs_to_datetime(b: divan::Bencher) {
    b.with_inputs(rand_secs).bench_refs(|i| datealgo::secs_to_datetime(divan::black_box(*i)));
}

#[divan::bench]
fn bench_datetime_to_secs(b: divan::Bencher) {
    b.with_inputs(rand_dt).bench_refs(|i| datealgo::datetime_to_secs(divan::black_box(*i)));
}

#[divan::bench]
fn bench_is_leap_year(b: divan::Bencher) {
    b.with_inputs(rand_year).bench_refs(|i| datealgo::is_leap_year(divan::black_box(*i)));
}

#[divan::bench]
fn bench_days_in_month(b: divan::Bencher) {
    b.with_inputs(rand_ym).bench_refs(|i| datealgo::days_in_month(divan::black_box(i.0), divan::black_box(i.1)));
}

#[divan::bench]
fn bench_rd_to_isoweekdate(b: divan::Bencher) {
    b.with_inputs(rand_rd).bench_refs(|i| datealgo::rd_to_isoweekdate(divan::black_box(*i)));
}

#[divan::bench]
fn bench_isoweekdate_to_rd(b: divan::Bencher) {
    b.with_inputs(rand_iwd).bench_refs(|i| datealgo::isoweekdate_to_rd(divan::black_box(*i)));
}

#[divan::bench]
fn bench_date_to_isoweekdate(b: divan::Bencher) {
    b.with_inputs(rand_date).bench_refs(|i| datealgo::date_to_isoweekdate(divan::black_box(*i)));
}

#[divan::bench]
fn bench_isoweekdate_to_date(b: divan::Bencher) {
    b.with_inputs(rand_iwd).bench_refs(|i| datealgo::isoweekdate_to_date(divan::black_box(*i)));
}

#[divan::bench]
fn bench_isoweeks_in_year(b: divan::Bencher) {
    b.with_inputs(rand_year).bench_refs(|i| datealgo::isoweeks_in_year(divan::black_box(*i)));
}

#[divan::bench]
fn bench_systemtime_to_secs(b: divan::Bencher) {
    b.with_inputs(rand_st).bench_refs(|i| datealgo::systemtime_to_secs(divan::black_box(*i)));
}

#[divan::bench]
fn bench_secs_to_systemtime(b: divan::Bencher) {
    b.with_inputs(rand_sn).bench_refs(|i| datealgo::secs_to_systemtime(divan::black_box(*i)));
}

#[divan::bench]
fn bench_systemtime_to_datetime(b: divan::Bencher) {
    b.with_inputs(rand_st).bench_refs(|i| datealgo::systemtime_to_datetime(divan::black_box(*i)));
}

#[divan::bench]
fn bench_datetime_to_systemtime(b: divan::Bencher) {
    b.with_inputs(rand_dtn).bench_refs(|i| datealgo::datetime_to_systemtime(divan::black_box(*i)));
}

fn main() {
    divan::main();
}
