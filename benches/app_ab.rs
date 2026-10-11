//! Application benchmark comparing two builds of datealgo with tango's paired
//! benchmarking, as the pull request check does. Run with
//! `scripts/app_ab.sh [GIT_REF]`.
//!
//! The `app` benchmark runs the synthetic application of `app_support.rs` with
//! datealgo, and `none` runs it without calendar work, so the time spent in
//! calendar code is the difference between them.

// Benchmarks need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use std::{
    process::ExitCode,
    sync::{Arc, OnceLock},
};
use tango_bench::{benchmark_fn, cli, Benchmark, IntoBenchmarks};

#[macro_use]
#[path = "app_support.rs"]
mod support;
use support::*;

#[path = "tango_support.rs"]
mod tango_support;
use tango_support::pin_worker;

fn app() -> Arc<App> {
    static APP: OnceLock<Arc<App>> = OnceLock::new();
    Arc::clone(APP.get_or_init(|| Arc::new(App::new())))
}

fn bench<C: Calendar + 'static>(name: &str) -> Benchmark {
    benchmark_fn(name, |b| {
        let app = app();
        let mut start = 0;
        let mut acc = 0;
        b.iter(move || {
            acc = app.run_placed::<C>(start, acc);
            start = (start + CHUNK) % EVENTS;
            acc
        })
    })
}

fn benchmarks() -> impl IntoBenchmarks {
    [bench::<Datealgo>("app"), bench::<NoCalendar>("none")]
}

fn main() -> cli::Result<ExitCode> {
    pin_worker();
    cli::run(benchmarks().into_benchmarks())
}
