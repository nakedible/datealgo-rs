#![allow(dead_code)]

use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

#[path = "basic_inputs.rs"]
mod inputs;
use inputs::*;

fn configure_basic_group(group: &mut BenchGroup) {
    group.config().max_rounds(20).max_time(Duration::from_millis(350));
}

fn bench_from_inputs<I, O>(group: &mut BenchGroup, name: &str, inputs: Arc<[I]>, mut run: impl FnMut(I) -> O + Send + 'static)
where
    I: Clone + Send + Sync + 'static,
{
    group.bench(name.to_owned(), move |b| {
        let inputs = Arc::clone(&inputs);
        let mut index = 0usize;
        b.iter(|| {
            let input = inputs[index & (INPUT_SET_SIZE - 1)].clone();
            index = index.wrapping_add(1);
            run(black_box(input))
        })
    });
}

fn bench_basic(suite: &mut Suite) {
    suite.group("overhead", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "overhead", inputs, black_box);
    });
    suite.group("rd_to_date", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_rd);
        bench_from_inputs(group, "rd_to_date", inputs, datealgo::rd_to_date);
    });
    suite.group("date_to_rd", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "date_to_rd", inputs, datealgo::date_to_rd);
    });
    suite.group("rd_to_weekday", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_rd);
        bench_from_inputs(group, "rd_to_weekday", inputs, datealgo::rd_to_weekday);
    });
    suite.group("date_to_weekday", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "date_to_weekday", inputs, datealgo::date_to_weekday);
    });
    suite.group("next_date", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "next_date", inputs, datealgo::next_date);
    });
    suite.group("prev_date", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "prev_date", inputs, datealgo::prev_date);
    });
    suite.group("secs_to_dhms", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_secs);
        bench_from_inputs(group, "secs_to_dhms", inputs, datealgo::secs_to_dhms);
    });
    suite.group("dhms_to_secs", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_dhms);
        bench_from_inputs(group, "dhms_to_secs", inputs, datealgo::dhms_to_secs);
    });
    suite.group("secs_to_datetime", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_secs);
        bench_from_inputs(group, "secs_to_datetime", inputs, datealgo::secs_to_datetime);
    });
    suite.group("datetime_to_secs", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_dt);
        bench_from_inputs(group, "datetime_to_secs", inputs, datealgo::datetime_to_secs);
    });
    suite.group("is_leap_year", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_year);
        bench_from_inputs(group, "is_leap_year", inputs, datealgo::is_leap_year);
    });
    suite.group("days_in_month", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_ym);
        bench_from_inputs(group, "days_in_month", inputs, |(y, m)| {
            datealgo::days_in_month(black_box(y), black_box(m))
        });
    });
    suite.group("rd_to_isoweekdate", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_rd);
        bench_from_inputs(group, "rd_to_isoweekdate", inputs, datealgo::rd_to_isoweekdate);
    });
    suite.group("isoweekdate_to_rd", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_iwd);
        bench_from_inputs(group, "isoweekdate_to_rd", inputs, datealgo::isoweekdate_to_rd);
    });
    suite.group("date_to_isoweekdate", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_date);
        bench_from_inputs(group, "date_to_isoweekdate", inputs, datealgo::date_to_isoweekdate);
    });
    suite.group("isoweekdate_to_date", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_iwd);
        bench_from_inputs(group, "isoweekdate_to_date", inputs, datealgo::isoweekdate_to_date);
    });
    suite.group("isoweeks_in_year", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_year);
        bench_from_inputs(group, "isoweeks_in_year", inputs, datealgo::isoweeks_in_year);
    });
    suite.group("systemtime_to_secs", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_st);
        bench_from_inputs(group, "systemtime_to_secs", inputs, datealgo::systemtime_to_secs);
    });
    suite.group("secs_to_systemtime", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_sn);
        bench_from_inputs(group, "secs_to_systemtime", inputs, datealgo::secs_to_systemtime);
    });
    suite.group("systemtime_to_datetime", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_st);
        bench_from_inputs(group, "systemtime_to_datetime", inputs, datealgo::systemtime_to_datetime);
    });
    suite.group("datetime_to_systemtime", |group| {
        configure_basic_group(group);
        let inputs = seeded_inputs(rand_dtn);
        bench_from_inputs(group, "datetime_to_systemtime", inputs, datealgo::datetime_to_systemtime);
    });
}

zenbench::main!(bench_basic);
