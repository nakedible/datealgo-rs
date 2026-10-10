use std::{sync::Arc, time::Duration};
use zenbench::prelude::*;

const N: usize = 262144;

macro_rules! candidates {
    ($group:ident, $inputs:ident) => {
        bench(&mut *$group, "datealgo", Arc::clone(&$inputs), datealgo::rd_to_date);
        bench(&mut *$group, "j64", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_j64);
        bench(&mut *$group, "j32", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_j32);
        bench(&mut *$group, "jm", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_jm);
        bench(&mut *$group, "a", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_a);
        bench(&mut *$group, "b2", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_b2);
        bench(&mut *$group, "c1", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_c1);
        bench(&mut *$group, "t", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_t);
        bench(&mut *$group, "t2", Arc::clone(&$inputs), datealgo::experiments::rd_to_date_t2);
    };
}

fn bench<O>(group: &mut BenchGroup, name: &str, inputs: Arc<[i32]>, f: fn(i32) -> O)
where
    O: 'static,
{
    group.bench(name.to_owned(), move |b| {
        let inputs = Arc::clone(&inputs);
        let mut i = 0usize;
        b.iter(|| {
            let x = inputs[i & (N - 1)];
            i = i.wrapping_add(1);
            f(black_box(x))
        })
    });
}

fn configure(group: &mut BenchGroup) {
    group.baseline("datealgo");
    group.config().max_rounds(30).max_time(Duration::from_millis(600)).baseline_only(true);
}

fn make(gen: impl FnMut(usize) -> i32) -> Arc<[i32]> {
    fastrand::seed(7);
    (0..N).map(gen).collect::<Vec<_>>().into()
}

fn exp(suite: &mut Suite) {
    let full = make(|_| fastrand::i32(datealgo::RD_MIN..=datealgo::RD_MAX));
    let lo = datealgo::date_to_rd((1970, 1, 1));
    let hi = datealgo::date_to_rd((2100, 12, 31));
    let modern = make(|_| fastrand::i32(lo..=hi));
    let start = datealgo::date_to_rd((2000, 1, 1));
    let seq = make(|i| start + i as i32);
    suite.group("rd_to_date_full", |g| {
        configure(g);
        candidates!(g, full);
    });
    suite.group("rd_to_date_modern", |g| {
        configure(g);
        candidates!(g, modern);
    });
    suite.group("rd_to_date_seq", |g| {
        configure(g);
        candidates!(g, seq);
    });
}

zenbench::main!(exp);
