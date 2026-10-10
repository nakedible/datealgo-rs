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

/// A named implementation, the current one or a candidate.
type Row<I, O> = (&'static str, fn(I) -> O);

fn bench<I, O>(group: &mut BenchGroup, fields: bool, name: &str, inputs: Arc<[I]>, f: fn(I) -> O)
where
    I: Copy + Send + Sync + 'static,
    O: Consume + 'static,
{
    group.bench(name.to_owned(), move |b| {
        let inputs = Arc::clone(&inputs);
        let mut i = 0usize;
        b.iter(|| {
            let x = inputs[i & (N - 1)];
            i = i.wrapping_add(1);
            if fields {
                f(black_box(x)).consume();
            } else {
                black_box(f(black_box(x)));
            }
        })
    });
}

/// Register one group per input distribution and consumption mode. The first
/// row is the baseline the others are compared against.
fn compare<I, O>(suite: &mut Suite, name: &str, distributions: &[Distribution<I>], rows: &[Row<I, O>])
where
    I: Copy + Send + Sync + 'static,
    O: Consume + 'static,
{
    for (dist, inputs) in distributions {
        for fields in [false, true] {
            let suffix = if fields { "_fields" } else { "" };
            suite.group(format!("{name}_{dist}{suffix}"), |group| {
                group.baseline(rows[0].0);
                group
                    .config()
                    .max_rounds(30)
                    .max_time(Duration::from_millis(600))
                    .baseline_only(true);
                for (row, f) in rows {
                    bench(group, fields, row, Arc::clone(inputs), *f);
                }
            });
        }
    }
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
    compare(
        suite,
        "rd_to_date",
        &rd_distributions(),
        &[("datealgo", datealgo::rd_to_date as fn(i32) -> (i32, u8, u8))],
    );
}

zenbench::main!(exp);
