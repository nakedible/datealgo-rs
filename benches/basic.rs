use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, SystemTime};

mod util;
use util::bencher;

fn rand_year() -> i32 {
    fastrand::i32(datealgo::YEAR_MIN..=datealgo::YEAR_MAX)
}

fn rand_rd() -> i32 {
    fastrand::i32(datealgo::RD_MIN..=datealgo::RD_MAX)
}

fn rand_date() -> (i32, i8, i8) {
    let y = rand_year();
    let m = fastrand::i8(1..=12);
    let d = fastrand::i8(1..=datealgo::days_in_month(y, m));
    (y, m, d)
}

fn rand_secs() -> i64 {
    fastrand::i64(datealgo::RD_SECONDS_MIN..=datealgo::RD_SECONDS_MAX)
}

fn rand_hms() -> (i8, i8, i8) {
    let h = fastrand::i8(0..=23);
    let m = fastrand::i8(0..=59);
    let s = fastrand::i8(0..=59);
    (h, m, s)
}

fn rand_dhms() -> (i32, i8, i8, i8) {
    let rd = rand_rd();
    let (h, m, s) = rand_hms();
    (rd, h, m, s)
}

fn rand_dt() -> (i32, i8, i8, i8, i8, i8) {
    let (y, m, d) = rand_date();
    let (hh, mm, ss) = rand_hms();
    (y, m, d, hh, mm, ss)
}

fn rand_ym() -> (i32, i8) {
    let y = rand_year();
    let m = fastrand::i8(1..=12);
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

fn rand_dtn() -> (i32, i8, i8, i8, i8, i8, u32) {
    let (y, m, d, hh, mm, ss) = rand_dt();
    let n = fastrand::u32(0..=999_999_999);
    (y, m, d, hh, mm, ss, n)
}

fn rand_iwd() -> (i32, i8, i8) {
    datealgo::rd_to_isoweekdate(rand_rd())
}

fn bench_basic(c: &mut Criterion) {
    c.bench_function("overhead", |b| {
        b.iter_custom(bencher(rand_date, |d| black_box(d)));
    });
    c.bench_function("rd_to_date", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_date(black_box(rd))))
    });
    c.bench_function("date_to_rd", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_rd(black_box(d))))
    });
    c.bench_function("rd_to_weekday", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_weekday(black_box(rd))))
    });
    c.bench_function("date_to_weekday", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_weekday(black_box(d))))
    });
    c.bench_function("next_date", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::next_date(black_box(d))))
    });
    c.bench_function("prev_date", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::prev_date(black_box(d))))
    });
    c.bench_function("secs_to_dhms", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_dhms(black_box(s))))
    });
    c.bench_function("dhms_to_secs", |b| {
        b.iter_custom(bencher(rand_dhms, |dhms| datealgo::dhms_to_secs(black_box(dhms))))
    });
    c.bench_function("secs_to_datetime", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_datetime(black_box(s))))
    });
    c.bench_function("datetime_to_secs", |b| {
        b.iter_custom(bencher(rand_dt, |dt| datealgo::datetime_to_secs(black_box(dt))))
    });
    c.bench_function("is_leap_year", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo::is_leap_year(black_box(y))))
    });
    c.bench_function("days_in_month", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo::days_in_month(black_box(y), black_box(m))))
    });
    c.bench_function("rd_to_isoweekdate", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_isoweekdate(black_box(rd))))
    });
    c.bench_function("isoweekdate_to_rd", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| datealgo::isoweekdate_to_rd(black_box(iwd))))
    });
    c.bench_function("date_to_isoweekdate", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_isoweekdate(black_box(d))))
    });
    c.bench_function("isoweekdate_to_date", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| datealgo::isoweekdate_to_date(black_box(iwd))))
    });
    c.bench_function("isoweeks_in_year", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo::isoweeks_in_year(black_box(y))))
    });
    c.bench_function("systemtime_to_secs", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_secs(black_box(st))))
    });
    c.bench_function("secs_to_systemtime", |b| {
        b.iter_custom(bencher(rand_sn, |sn| datealgo::secs_to_systemtime(black_box(sn))))
    });
    c.bench_function("systemtime_to_datetime", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_datetime(black_box(st))))
    });
    c.bench_function("datetime_to_systemtime", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| datealgo::datetime_to_systemtime(black_box(dtn))))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(5000).measurement_time(Duration::from_secs(10));
    targets = bench_basic
}
criterion_main!(benches);
