//! The basic benchmarks of each function on their own, for comparing two
//! builds of datealgo with tango's paired benchmarking, as the pull request
//! check does. Run with `scripts/basic_ab.sh [GIT_REF]`.

// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use std::{hint::black_box, process::ExitCode};
use tango_bench::{benchmark_fn, cli, Benchmark, IntoBenchmarks};

#[path = "basic_inputs.rs"]
mod inputs;
use inputs::*;

#[path = "tango_support.rs"]
mod tango_support;
use tango_support::pin_worker;

/// Number of calls per iteration. Results are reported per call.
const BATCH: usize = 64;
const _: () = assert!(INPUT_SET_SIZE % BATCH == 0);

/// A benchmark of calling `f` with inputs from `generate`. `f` is a distinct
/// type for each benchmark, so it is inlined as it would be in a caller.
fn bench<I, O, F>(name: &str, generate: fn() -> I, f: F) -> Benchmark
where
    I: Copy + 'static,
    O: 'static,
    F: Fn(I) -> O + Copy + 'static,
{
    benchmark_fn(name, move |b| {
        let inputs = seeded_inputs(generate);
        let mut i = 0;
        b.iter(move || {
            // Tango keeps the state of this closure in memory between calls,
            // so each call runs a batch of calls from locals. The inputs are
            // runtime data, so they need no black_box.
            let start = i & (INPUT_SET_SIZE - 1);
            i += BATCH;
            for &x in &inputs[start..start + BATCH] {
                black_box(f(x));
            }
        })
    })
}

fn benchmarks() -> impl IntoBenchmarks {
    [
        bench("overhead", rand_date, |d| d),
        bench("rd_to_date", rand_rd, datealgo::rd_to_date),
        bench("date_to_rd", rand_date, datealgo::date_to_rd),
        bench("rd_to_weekday", rand_rd, datealgo::rd_to_weekday),
        bench("date_to_weekday", rand_date, datealgo::date_to_weekday),
        bench("next_date", rand_date, datealgo::next_date),
        bench("prev_date", rand_date, datealgo::prev_date),
        bench("secs_to_dhms", rand_secs, datealgo::secs_to_dhms),
        bench("dhms_to_secs", rand_dhms, datealgo::dhms_to_secs),
        bench("secs_to_datetime", rand_secs, datealgo::secs_to_datetime),
        bench("datetime_to_secs", rand_dt, datealgo::datetime_to_secs),
        bench("is_leap_year", rand_year, datealgo::is_leap_year),
        bench("days_in_month", rand_ym, |(y, m)| datealgo::days_in_month(y, m)),
        bench("rd_to_isoweekdate", rand_rd, datealgo::rd_to_isoweekdate),
        bench("isoweekdate_to_rd", rand_iwd, datealgo::isoweekdate_to_rd),
        bench("date_to_isoweekdate", rand_date, datealgo::date_to_isoweekdate),
        bench("isoweekdate_to_date", rand_iwd, datealgo::isoweekdate_to_date),
        bench("isoweeks_in_year", rand_year, datealgo::isoweeks_in_year),
        bench("systemtime_to_secs", rand_st, datealgo::systemtime_to_secs),
        bench("secs_to_systemtime", rand_sn, datealgo::secs_to_systemtime),
        bench("systemtime_to_datetime", rand_st, datealgo::systemtime_to_datetime),
        bench("datetime_to_systemtime", rand_dtn, datealgo::datetime_to_systemtime),
    ]
}

fn main() -> cli::Result<ExitCode> {
    pin_worker();
    cli::run(benchmarks().into_benchmarks())
}
