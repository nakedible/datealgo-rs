//! The synthetic application of the application benchmarks.
//!
//! The application is split into domains, each a different kind of software
//! that uses a date library, from a study of how the most used crates that
//! depend on chrono, time and jiff use them: logging, web and messaging
//! formats, databases and columnar engines, calendar arithmetic, weeks,
//! scheduling and time zones, and files, certificates and documents. Each
//! domain processes a fixed trace of events, each one thing that kind of
//! software does with dates, with inputs from distributions like those it sees,
//! mostly near the present. Each kind of event comes in several
//! implementations, written the way different libraries and applications write
//! them, as the code around the calendar functions affects how they run.
//!
//! Each event is handled by a function that is not inlined, with the calendar
//! code inlined into it as at a call site in user code, and between events the
//! rest of the application runs: reads over a working set larger than L2, and
//! code and branches of its own. Each event waits for the result of the
//! previous one, so calls are not overlapped more than in a real program.
//!
//! The application runs with an implementation of `Calendar`. The
//! `NoCalendar` implementation does no calendar work, so the difference to it
//! is the cost of the calendar code in context.

#![allow(dead_code)]

pub mod arith;
pub mod calendar;
pub mod data;
pub mod files;
pub mod logging;
pub mod schedule;
pub mod text;
pub mod weeks;
pub mod wire;

pub use calendar::*;

use std::sync::OnceLock;

/// A domain of the application: a trace of events and their handlers
pub trait Workload: Send + Sync + 'static {
    /// Name of the domain, used as the name of its benchmark
    const NAME: &'static str;

    /// Weight of the domain in the overall score, in percent
    const WEIGHT: u32;

    /// Generate the trace with the given random number generator
    fn new(rng: &mut fastrand::Rng) -> Self;

    /// Handle event `i` of the trace, with its input changed by `dep`, which
    /// depends on the previous result but is always 0, and return a hash of
    /// the result
    fn handle<C: Calendar>(&self, i: usize, dep: u64) -> u64;
}

/// Calls each domain's function with the domain's type, in the order of their
/// weights
#[macro_export]
macro_rules! for_each_domain {
    ($f:ident) => {
        $f!(logging, logging::Logging);
        $f!(wire, wire::Wire);
        $f!(data, data::Data);
        $f!(arith, arith::Arith);
        $f!(weeks, weeks::Weeks);
        $f!(schedule, schedule::Schedule);
        $f!(files, files::Files);
    };
}

/// Events in each trace, replayed in a loop
pub const EVENTS: usize = 1 << 16;

/// Events per timed iteration
pub const CHUNK: usize = 256;

/// The trace of a domain, generated once per process
pub fn workload<W: Workload>() -> &'static W {
    // A static in a generic function is shared by all instances, so each
    // domain is boxed in a map by its name.
    static MAP: OnceLock<std::sync::Mutex<std::collections::HashMap<&'static str, &'static (dyn std::any::Any + Send + Sync)>>> =
        OnceLock::new();
    let map = MAP.get_or_init(Default::default);
    let mut map = map.lock().unwrap();
    let any = *map.entry(W::NAME).or_insert_with(|| {
        let mut rng = fastrand::Rng::with_seed(7);
        Box::leak(Box::new(W::new(&mut rng)))
    });
    any.downcast_ref::<W>().unwrap()
}

// The rest of the application

/// Words in the working set of the rest of the application, 4 MiB
pub const WORKING_SET: usize = 1 << 19;

/// Some of the rest of the application, reading `lines` cache lines from its
/// working set, with a branch on each value read. Each instance has code and
/// branches of its own.
#[inline(never)]
pub fn other<const K: u64>(mem: &[u64], seed: u64, lines: u32) -> u64 {
    let mut x = seed | 1;
    let mut acc = K;
    for _ in 0..lines {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let v = mem[(x as usize) & (WORKING_SET - 8)];
        if v >> (K % 61) & 1 != 0 {
            acc = acc.wrapping_add(v ^ K);
        } else {
            acc = acc.rotate_left((K % 31) as u32 + 1) ^ v;
        }
    }
    acc
}

macro_rules! others {
    ($($k:literal)*) => { [$(other::<$k>,)*] };
}

pub type Other = fn(&[u64], u64, u32) -> u64;
pub const OTHERS: [Other; 64] = others!(
    0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

/// Cache lines read by the rest of the application between two events, and
/// the probability of each in percent
pub const OTHER_LINES: [(u16, u32); 4] = [(4, 60), (16, 30), (64, 8), (1024, 2)];

/// The rest of the application: its working set, and which of its functions
/// runs between two events and how much it reads, the same for all domains
pub struct Rest {
    pub mem: Vec<u64>,
    pub other: Vec<(u8, u16)>,
}

impl Rest {
    pub fn get() -> &'static Rest {
        static REST: OnceLock<Rest> = OnceLock::new();
        REST.get_or_init(|| {
            let mut rng = fastrand::Rng::with_seed(11);
            Rest {
                mem: (0..WORKING_SET).map(|_| rng.u64(..)).collect(),
                other: (0..EVENTS)
                    .map(|_| (rng.u8(0..OTHERS.len() as u8), pick(&mut rng, &OTHER_LINES)))
                    .collect(),
            }
        })
    }
}

/// Process `CHUNK` events of a domain from `start`, continuing from the hash
/// `acc` of the previous results
pub fn run<C: Calendar, W: Workload>(w: &W, rest: &Rest, start: usize, mut acc: u64) -> u64 {
    for i in start..start + CHUNK {
        // Make the input wait for the previous result, as in a real program,
        // with a value that is zero unless the result happens to be all ones,
        // so that the inputs are the same for all implementations.
        let h = w.handle::<C>(i, (acc == u64::MAX) as u64);
        acc = acc.rotate_left(7) ^ h;
        // The rest of the application waits for the result in the same way,
        // and its work depends only on the event, so that it is the same for
        // all implementations, even those whose results differ.
        let (f, lines) = rest.other[i];
        let seed = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (acc == u64::MAX) as u64;
        acc ^= OTHERS[f as usize](&rest.mem, seed, lines as u32);
    }
    acc
}

/// Process `CHUNK` events like `run`, with one of the placed copies of the
/// implementation in turn
pub fn run_placed<C: Calendar, W: Workload>(w: &W, rest: &Rest, start: usize, acc: u64) -> u64 {
    match start / CHUNK % 8 {
        0 => run::<Placed<C, 0>, W>(w, rest, start, acc),
        1 => run::<Placed<C, 1>, W>(w, rest, start, acc),
        2 => run::<Placed<C, 2>, W>(w, rest, start, acc),
        3 => run::<Placed<C, 3>, W>(w, rest, start, acc),
        4 => run::<Placed<C, 4>, W>(w, rest, start, acc),
        5 => run::<Placed<C, 5>, W>(w, rest, start, acc),
        6 => run::<Placed<C, 6>, W>(w, rest, start, acc),
        _ => run::<Placed<C, 7>, W>(w, rest, start, acc),
    }
}

/// Hash of the results of the whole trace of a domain
pub fn checksum<C: Calendar, W: Workload>() -> u64 {
    let (w, rest) = (workload::<W>(), Rest::get());
    (0..EVENTS)
        .step_by(CHUNK)
        .fold(0, |acc, start| run_placed::<C, W>(w, rest, start, acc))
}

// Inputs

/// The present of the traces, 2026-10-10T00:00:00Z
pub const NOW: i64 = 1_791_590_400;
pub const DAY: i64 = 86_400;
pub const YEAR: i64 = 365 * DAY;
const _: () = assert!(NOW == 20_736 * DAY);

/// One of the given choices, by their weights
pub fn pick<T: Copy>(rng: &mut fastrand::Rng, choices: &[(T, u32)]) -> T {
    let mut r = rng.u32(0..choices.iter().map(|c| c.1).sum());
    for &(v, p) in choices {
        if r < p {
            return v;
        }
        r -= p;
    }
    unreachable!()
}

/// A time within `span` seconds of the present, in the past or future
pub fn near_now(rng: &mut fastrand::Rng, span: i64) -> i64 {
    NOW + rng.i64(-span..=span)
}

/// A date of the kind data mostly holds: near the present, birth dates, and
/// rarely anything in 1900-2100 or the edges of 0001-9999
pub fn data_date(rng: &mut fastrand::Rng) -> (i32, u8, u8) {
    let secs = match rng.u32(0..100) {
        0..=59 => near_now(rng, 3 * YEAR),
        60..=79 => NOW - rng.i64(15 * YEAR..85 * YEAR),
        80..=97 => rng.i64(-70 * YEAR..130 * YEAR),
        _ => rng.i64(-62_135_596_800..=253_402_214_400),
    };
    datealgo::rd_to_date(secs.div_euclid(DAY) as i32)
}
