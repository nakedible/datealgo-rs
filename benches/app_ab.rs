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

/// Pin a tango worker process to the CPU given for its role in
/// `APP_AB_CPU_CANDIDATE` or `APP_AB_CPU_BASELINE`. Tango runs the two builds
/// at the same time on different CPUs, and CPUs can differ in speed, so
/// `scripts/app_ab.sh` chooses which build runs on which CPU, and swaps them.
fn pin_worker() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("__worker") {
        return;
    }
    let Some(role) = args.windows(2).find(|w| w[0] == "--role").map(|w| w[1].to_uppercase()) else {
        return;
    };
    let Ok(cpu) = std::env::var(format!("APP_AB_CPU_{role}")) else {
        return;
    };
    let cpu: usize = cpu.parse().expect("CPU number");
    #[cfg(target_os = "linux")]
    // SAFETY: the set is initialized before it is passed to the kernel
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_SET(cpu, &mut set);
        let ret = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
        assert_eq!(ret, 0, "could not pin to CPU {cpu}");
    }
}

fn main() -> cli::Result<ExitCode> {
    pin_worker();
    cli::run(benchmarks().into_benchmarks())
}
