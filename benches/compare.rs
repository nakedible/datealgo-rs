use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use std::time::Duration;

mod util;
use util::bencher;

include!("compare_support.rs");

fn bench_rd_to_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_rd_to_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_date(black_box(rd))))
    });
    group.bench_function("hinnant_unsigned", |b| {
        b.iter_custom(bencher(rand_rd, |rd| hinnant::civil_from_days_u(black_box(rd))))
    });
    group.bench_function("hinnant", |b| {
        b.iter_custom(bencher(rand_rd, |rd| hinnant::civil_from_days(black_box(rd))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_rd, |rd| httpdate::rd_to_date(black_box(rd))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_rd, |rd| humantime::rd_to_date(black_box(rd))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_rd, |rd| chrono::rd_to_date(black_box(rd))))
    });
    group.bench_function("time", |b| b.iter_custom(bencher(rand_rd, |rd| time::rd_to_date(black_box(rd)))));
    group.finish();
}

fn bench_date_to_rd(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_rd");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_rd(black_box(d))))
    });
    group.bench_function("hinnant_unsigned", |b| {
        b.iter_custom(bencher(rand_date, |d| hinnant::days_from_civil_u(black_box(d))))
    });
    group.bench_function("hinnant", |b| {
        b.iter_custom(bencher(rand_date, |d| hinnant::days_from_civil(black_box(d))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_date, |d| httpdate::date_to_rd(black_box(d))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_date, |d| humantime::date_to_rd(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_date, |d| chrono::date_to_rd(black_box(d))))
    });
    group.bench_function("time", |b| b.iter_custom(bencher(rand_date, |d| time::date_to_rd(black_box(d)))));
    group.finish();
}

fn bench_rd_to_weekday(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_rd_to_weekday");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_weekday(black_box(rd))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo_alt::rd_to_weekday(black_box(rd))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo_alt::rd_to_weekday2(black_box(rd))))
    });
    group.bench_function("datealgo_alt3", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo_alt::rd_to_weekday3(black_box(rd))))
    });
    group.finish();
}

fn bench_date_to_weekday(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_weekday");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_weekday(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::date_to_weekday(black_box(d))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::date_to_weekday2(black_box(d))))
    });
    group.finish();
}

fn bench_next_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_next_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::next_date(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::next_date(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(chrono::rand_date, |d| chrono::next_date(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(time::rand_date, |d| time::next_date(black_box(d))))
    });
    group.finish();
}

fn bench_prev_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_prev_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::prev_date(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::prev_date(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(chrono::rand_date, |d| chrono::prev_date(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(time::rand_date, |d| time::prev_date(black_box(d))))
    });
    group.finish();
}

fn bench_secs_to_dhms(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_dhms");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_dhms(black_box(s))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo_alt::secs_to_dhms(black_box(s))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo_alt::secs_to_dhms2(black_box(s))))
    });
    group.finish();
}

fn bench_dhms_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_dhms_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dhms, |dhms| datealgo::dhms_to_secs(black_box(dhms))))
    });
    group.finish();
}

fn bench_secs_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_datetime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_datetime(black_box(s))))
    });
    group.finish();
}

fn bench_datetime_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_datetime_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dt, |dt| datealgo::datetime_to_secs(black_box(dt))))
    });
    group.finish();
}

fn bench_is_leap_year(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_is_leap_year");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo::is_leap_year(black_box(y))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year(black_box(y))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year2(black_box(y))))
    });
    group.bench_function("datealgo_alt3", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year3(black_box(y))))
    });
    group.bench_function("datealgo_alt4", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year4(black_box(y))))
    });
    group.finish();
}

fn bench_days_in_month(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_days_in_month");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo::days_in_month(black_box(y), black_box(m))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo_alt::days_in_month(black_box(y), black_box(m))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo_alt::days_in_month2(black_box(y), black_box(m))))
    });
    group.finish();
}

fn bench_date_to_isoweekdate(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_isoweekdate");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_isoweekdate(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_date, |d| chrono::date_to_isoweekdate(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_date, |d| time::date_to_isoweekdate(black_box(d))))
    });
    group.finish();
}

fn bench_isoweekdate_to_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_isoweekdate_to_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| datealgo::isoweekdate_to_date(black_box(iwd))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| chrono::isoweekdate_to_date(black_box(iwd))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| time::isoweekdate_to_date(black_box(iwd))))
    });
    group.finish();
}

fn bench_systemtime_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_systemtime_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_secs(black_box(st))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo_alt::systemtime_to_secs(black_box(st))))
    });
    group.finish();
}

fn bench_secs_to_systemtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_systemtime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_sn, |sn| datealgo::secs_to_systemtime(black_box(sn))))
    });
    group.finish();
}

fn bench_systemtime_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_systemtime_to_datetime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_st, |st| httpdate::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_st, |st| humantime::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_st, |st| time::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_st, |st| chrono::systemtime_to_datetime(black_box(st))))
    });
    group.finish();
}

fn bench_datetime_to_systemtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_datetime_to_systemtime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| datealgo::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| httpdate::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| humantime::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| time::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| chrono::datetime_to_systemtime(black_box(dtn))))
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(5000).measurement_time(Duration::from_secs(10));
    targets =
        bench_rd_to_date,
        bench_date_to_rd,
        bench_rd_to_weekday,
        bench_date_to_weekday,
        bench_next_date,
        bench_prev_date,
        bench_secs_to_dhms,
        bench_dhms_to_secs,
        bench_secs_to_datetime,
        bench_datetime_to_secs,
        bench_is_leap_year,
        bench_days_in_month,
        bench_date_to_isoweekdate,
        bench_isoweekdate_to_date,
        bench_systemtime_to_secs,
        bench_secs_to_systemtime,
        bench_systemtime_to_datetime,
        bench_datetime_to_systemtime,
);
criterion_main!(benches);
