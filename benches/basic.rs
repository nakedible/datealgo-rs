use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, UNIX_EPOCH};

fn bench_basic(c: &mut Criterion) {
    let rd = datealgo::date_to_rd((2023, 5, 12));
    c.bench_function("rd_to_date", |b| b.iter(|| black_box(datealgo::rd_to_date(black_box(rd)))));
    let d = (2023, 5, 12);
    c.bench_function("date_to_rd", |b| b.iter(|| black_box(datealgo::date_to_rd(black_box(d)))));
    c.bench_function("rd_to_weekday", |b| b.iter(|| black_box(datealgo::rd_to_weekday(black_box(rd)))));
    c.bench_function("date_to_weekday", |b| b.iter(|| black_box(datealgo::date_to_weekday(black_box(d)))));
    let s = 1684574678i64;
    c.bench_function("secs_to_dhms", |b| b.iter(|| black_box(datealgo::secs_to_dhms(black_box(s)))));
    let dhms = (123123, 12, 34, 56);
    c.bench_function("dhms_to_secs", |b| b.iter(|| black_box(datealgo::dhms_to_secs(black_box(dhms)))));
    c.bench_function("secs_to_datetime", |b| {
        b.iter(|| black_box(datealgo::secs_to_datetime(black_box(s))))
    });
    let dt = (2023, 5, 20, 12, 34, 56);
    c.bench_function("datetime_to_secs", |b| {
        b.iter(|| black_box(datealgo::datetime_to_secs(black_box(dt))))
    });
    let y = 2000;
    c.bench_function("is_leap_year", |b| b.iter(|| black_box(datealgo::is_leap_year(black_box(y)))));
    let m = 2;
    c.bench_function("days_in_month", |b| {
        b.iter(|| black_box(datealgo::days_in_month(black_box(y), black_box(m))))
    });
    let st = UNIX_EPOCH + Duration::from_secs(1684574678);
    c.bench_function("systemtime_to_secs", |b| {
        b.iter(|| black_box(datealgo::systemtime_to_secs(black_box(st))))
    });
    let sn = (1684574678, 0);
    c.bench_function("secs_to_systemtime", |b| {
        b.iter(|| black_box(datealgo::secs_to_systemtime(black_box(sn))))
    });
    c.bench_function("systemtime_to_datetime", |b| {
        b.iter(|| black_box(datealgo::systemtime_to_datetime(black_box(st))))
    });
    let dtn = (2023, 5, 20, 12, 34, 56, 0);
    c.bench_function("datetime_to_systemtime", |b| {
        b.iter(|| black_box(datealgo::datetime_to_systemtime(black_box(dtn))))
    });
}

criterion_group!(benches, bench_basic,);
criterion_main!(benches);
