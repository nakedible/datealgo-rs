//! Application benchmark comparing two builds of datealgo with tango's paired
//! benchmarking, as the pull request check does. Run with
//! `scripts/app_ab.sh [GIT_REF]`.
//!
//! Each domain of the synthetic application of `benches/app/` has a benchmark
//! with datealgo, and one without calendar work, named with `_none`, so the
//! time spent in calendar code is the difference between them.

// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use std::process::ExitCode;
use tango_bench::{benchmark_fn, cli, Benchmark};

#[macro_use]
#[path = "app/mod.rs"]
mod app;
use app::*;

#[path = "tango_support.rs"]
mod tango_support;
use tango_support::pin_worker;

fn bench<C: Calendar + 'static, W: Workload>(name: String) -> Benchmark {
    benchmark_fn(name, |b| {
        let (w, rest) = (workload::<W>(), Rest::get());
        let mut start = 0;
        let mut acc = 0;
        b.iter(move || {
            acc = run_placed::<C, W>(w, rest, start, acc);
            start = (start + CHUNK) % EVENTS;
            acc
        })
    })
}

fn benchmarks() -> Vec<Benchmark> {
    let mut benchmarks = Vec::new();
    macro_rules! register {
        ($m:ident, $t:ty) => {
            benchmarks.push(bench::<Datealgo, $t>(<$t>::NAME.to_owned()));
            benchmarks.push(bench::<NoCalendar, $t>(format!("{}_none", <$t>::NAME)));
        };
    }
    for_each_domain!(register);
    benchmarks
}

fn main() -> cli::Result<ExitCode> {
    pin_worker();
    cli::run(benchmarks())
}
