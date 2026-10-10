//! Experiment suite: compares candidate implementations from
//! `datealgo::experiments` against the current ones, interleaved in the same
//! rounds. Run with `scripts/exp_bench.sh`.
//!
//! On the main branch each function only has its `datealgo` row. To evaluate a
//! candidate, add a row for it next to the `datealgo` row in `exp()`.

use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

/// Number of inputs replayed in a loop, large enough that the branch predictor
/// can't learn the sequence. A power of two, so inputs can be indexed with a
/// mask.
const N: usize = 262144;
const _: () = assert!(N.is_power_of_two());

/// Number of calls per timed iteration. Results are reported per call.
const BATCH: usize = 64;
const _: () = assert!(N % BATCH == 0);

/// Consume a result field by field, as a caller building its own
/// representation would, instead of as one packed value.
trait Consume {
    fn consume(self);
}

macro_rules! consume_tuple {
    ($($t:ident $i:tt),+) => {
        impl Consume for ($($t,)+) {
            fn consume(self) {
                $(black_box(self.$i);)+
            }
        }
    };
}

consume_tuple!(i32 0, u8 1, u8 2);
consume_tuple!(i32 0, u8 1, u8 2, u8 3);
consume_tuple!(i32 0, u8 1, u8 2, u8 3, u8 4, u8 5);

macro_rules! consume_scalar {
    ($($t:ty),+) => {
        $(impl Consume for $t {
            fn consume(self) {
                black_box(self);
            }
        })+
    };
}

consume_scalar!(i32, i64, u8, bool);

/// A named input set.
type Distribution<I> = (&'static str, Arc<[I]>);

fn bench<I, O, F>(group: &mut BenchGroup, fields: bool, name: &str, inputs: Arc<[I]>, f: F)
where
    I: Copy + Send + Sync + 'static,
    O: Consume + 'static,
    F: Fn(I) -> O + Copy + Send + 'static,
{
    group.bench(name.to_owned(), move |b| {
        let inputs = Arc::clone(&inputs);
        let mut i = 0usize;
        b.iter(move || {
            // The timing loop calls this closure without inlining it, so its
            // state lives in memory, and updating it on every call would add
            // a store and a reload to each measurement. A batch per call runs
            // from registers instead. `f` is a distinct type for each row, so
            // it is inlined as in a caller. The inputs are runtime data, so
            // they need no black_box. Leaving it out lets tuples reach the
            // function field by field instead of packed into one register.
            let start = i & (N - 1);
            i = i.wrapping_add(BATCH);
            for &x in &inputs[start..start + BATCH] {
                if fields {
                    f(x).consume();
                } else {
                    black_box(f(x));
                }
            }
        })
    });
}

/// Register one group per input distribution and consumption mode, with one
/// row per implementation. The first row is the baseline the others are
/// compared against.
macro_rules! compare {
    ($suite:expr, $name:expr, $distributions:expr, [$(($row:expr, $f:expr)),+ $(,)?]) => {
        for (dist, inputs) in $distributions {
            for fields in [false, true] {
                let suffix = if fields { "_fields" } else { "" };
                $suite.group(format!("{}_{dist}{suffix}", $name), |group| {
                    group.baseline([$($row),+][0]);
                    group
                        .config()
                        .max_rounds(30)
                        .max_time(Duration::from_millis(600))
                        .baseline_only(true);
                    group.throughput(Throughput::Elements(BATCH as u64)).throughput_unit("calls");
                    $(bench(group, fields, $row, Arc::clone(&inputs), $f);)+
                });
            }
        }
    };
}

fn inputs<I>(generate: impl FnMut(usize) -> I) -> Arc<[I]> {
    fastrand::seed(7);
    (0..N).map(generate).collect::<Vec<_>>().into()
}

/// Rata die inputs: uniform over the whole supported range, uniform over
/// 1970-2100, and consecutive days from 2000-01-01.
fn rd_distributions() -> Vec<Distribution<i32>> {
    let lo = datealgo::date_to_rd((1970, 1, 1));
    let hi = datealgo::date_to_rd((2100, 12, 31));
    let start = datealgo::date_to_rd((2000, 1, 1));
    vec![
        ("full", inputs(|_| fastrand::i32(datealgo::RD_MIN..=datealgo::RD_MAX))),
        ("modern", inputs(|_| fastrand::i32(lo..=hi))),
        ("seq", inputs(|i| start + i as i32)),
    ]
}

fn exp(suite: &mut Suite) {
    use datealgo::experiments as x;
    let years: Vec<Distribution<i32>> = vec![
        ("full", inputs(|_| fastrand::i32(datealgo::YEAR_MIN..=datealgo::YEAR_MAX))),
        ("modern", inputs(|_| fastrand::i32(1970..=2100))),
    ];
    compare!(
        suite,
        "is_leap_year",
        years,
        [
            ("datealgo", datealgo::is_leap_year),
            ("h64", x::is_leap_year_h64),
            ("abs", x::is_leap_year_abs),
        ]
    );
    compare!(
        suite,
        "rd_to_date",
        rd_distributions(),
        [("datealgo", datealgo::rd_to_date), ("table", x::rd_to_date_table)]
    );
    let dates: Vec<Distribution<(i32, u8, u8)>> = rd_distributions()
        .into_iter()
        .map(|(name, rds)| (name, rds.iter().map(|&n| datealgo::rd_to_date(n)).collect::<Vec<_>>().into()))
        .collect();
    compare!(
        suite,
        "date_to_rd",
        dates,
        [("datealgo", datealgo::date_to_rd), ("table", x::date_to_rd_table)]
    );
}

zenbench::main!(exp);
