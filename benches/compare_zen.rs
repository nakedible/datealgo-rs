#![allow(dead_code)]

use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

include!("compare_support.rs");

const INPUT_SET_SIZE: usize = 4096;

fn configure_compare_group(group: &mut BenchGroup) {
    group.baseline("datealgo");
    group
        .config()
        .max_rounds(20)
        .max_time(Duration::from_millis(350))
        .baseline_only(true)
        .sort_by_speed(true);
}

fn seeded_inputs<I>(mut generate: impl FnMut() -> I) -> Arc<[I]> {
    fastrand::seed(7);
    (0..INPUT_SET_SIZE).map(|_| generate()).collect::<Vec<_>>().into()
}

fn bench_from_inputs<I, O>(group: &mut BenchGroup, name: &str, inputs: Arc<[I]>, mut run: impl FnMut(I) -> O + Send + 'static)
where
    I: Clone + Send + Sync + 'static,
{
    group.bench(name.to_owned(), move |b| {
        let inputs = Arc::clone(&inputs);
        let mut index = 0usize;
        b.iter(|| {
            let input = inputs[index % inputs.len()].clone();
            index = index.wrapping_add(1);
            run(black_box(input))
        })
    });
}

fn bench_rd_to_date(suite: &mut Suite) {
    suite.group("compare_rd_to_date", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_rd);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::rd_to_date);
        bench_from_inputs(group, "hinnant_unsigned", Arc::clone(&inputs), hinnant::civil_from_days_u);
        bench_from_inputs(group, "hinnant", Arc::clone(&inputs), hinnant::civil_from_days);
        bench_from_inputs(group, "httpdate", Arc::clone(&inputs), httpdate::rd_to_date);
        bench_from_inputs(group, "humantime", Arc::clone(&inputs), humantime::rd_to_date);
        bench_from_inputs(group, "chrono", Arc::clone(&inputs), chrono::rd_to_date);
        bench_from_inputs(group, "time", inputs, time::rd_to_date);
    });
}

fn bench_date_to_rd(suite: &mut Suite) {
    suite.group("compare_date_to_rd", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::date_to_rd);
        bench_from_inputs(group, "hinnant_unsigned", Arc::clone(&inputs), hinnant::days_from_civil_u);
        bench_from_inputs(group, "hinnant", Arc::clone(&inputs), hinnant::days_from_civil);
        bench_from_inputs(group, "httpdate", Arc::clone(&inputs), httpdate::date_to_rd);
        bench_from_inputs(group, "humantime", Arc::clone(&inputs), humantime::date_to_rd);
        bench_from_inputs(group, "chrono", Arc::clone(&inputs), chrono::date_to_rd);
        bench_from_inputs(group, "time", inputs, time::date_to_rd);
    });
}

fn bench_next_date(suite: &mut Suite) {
    suite.group("compare_next_date", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::next_date);
        bench_from_inputs(group, "datealgo_alt", Arc::clone(&inputs), datealgo_alt::next_date);

        let chrono_inputs = seeded_inputs(chrono::rand_date);
        bench_from_inputs(group, "chrono", chrono_inputs, chrono::next_date);

        let time_inputs = seeded_inputs(time::rand_date);
        bench_from_inputs(group, "time", time_inputs, time::next_date);
    });
}

fn bench_prev_date(suite: &mut Suite) {
    suite.group("compare_prev_date", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::prev_date);
        bench_from_inputs(group, "datealgo_alt", Arc::clone(&inputs), datealgo_alt::prev_date);

        let chrono_inputs = seeded_inputs(chrono::rand_date);
        bench_from_inputs(group, "chrono", chrono_inputs, chrono::prev_date);

        let time_inputs = seeded_inputs(time::rand_date);
        bench_from_inputs(group, "time", time_inputs, time::prev_date);
    });
}

fn bench_date_to_isoweekdate(suite: &mut Suite) {
    suite.group("compare_date_to_isoweekdate", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::date_to_isoweekdate);
        bench_from_inputs(group, "chrono", Arc::clone(&inputs), chrono::date_to_isoweekdate);
        bench_from_inputs(group, "time", inputs, time::date_to_isoweekdate);
    });
}

fn bench_isoweekdate_to_date(suite: &mut Suite) {
    suite.group("compare_isoweekdate_to_date", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_iwd);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::isoweekdate_to_date);
        bench_from_inputs(group, "chrono", Arc::clone(&inputs), chrono::isoweekdate_to_date);
        bench_from_inputs(group, "time", inputs, time::isoweekdate_to_date);
    });
}

fn bench_systemtime_to_datetime(suite: &mut Suite) {
    suite.group("compare_systemtime_to_datetime", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_st);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::systemtime_to_datetime);
        bench_from_inputs(group, "httpdate", Arc::clone(&inputs), httpdate::systemtime_to_datetime);
        bench_from_inputs(group, "humantime", Arc::clone(&inputs), humantime::systemtime_to_datetime);
        bench_from_inputs(group, "time", Arc::clone(&inputs), time::systemtime_to_datetime);
        bench_from_inputs(group, "chrono", inputs, chrono::systemtime_to_datetime);
    });
}

fn bench_datetime_to_systemtime(suite: &mut Suite) {
    suite.group("compare_datetime_to_systemtime", |group| {
        configure_compare_group(group);

        let inputs = seeded_inputs(rand_dtn);
        bench_from_inputs(group, "datealgo", Arc::clone(&inputs), datealgo::datetime_to_systemtime);
        bench_from_inputs(group, "httpdate", Arc::clone(&inputs), httpdate::datetime_to_systemtime);
        bench_from_inputs(group, "humantime", Arc::clone(&inputs), humantime::datetime_to_systemtime);
        bench_from_inputs(group, "time", Arc::clone(&inputs), time::datetime_to_systemtime);
        bench_from_inputs(group, "chrono", inputs, chrono::datetime_to_systemtime);
    });
}

zenbench::main!(
    bench_rd_to_date,
    bench_date_to_rd,
    bench_next_date,
    bench_prev_date,
    bench_date_to_isoweekdate,
    bench_isoweekdate_to_date,
    bench_systemtime_to_datetime,
    bench_datetime_to_systemtime,
);
