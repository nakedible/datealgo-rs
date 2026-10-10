use std::{
    hint::black_box,
    time::{Duration, Instant},
};

// Large enough that the branch predictor can't learn the input sequence. With
// a few thousand inputs replayed millions of times, code with data-dependent
// branches measured about twice as fast as on unpredictable inputs, and some
// branches were still partly learned with 65536 inputs. The inputs no longer
// fit in L2, but sequential access lets the prefetcher hide that. A power of
// two, so inputs can be indexed with a mask.
const ARR_SIZE: usize = 262144;
const _: () = assert!(ARR_SIZE.is_power_of_two());

pub fn bencher<I: Copy, O>(s: impl Fn() -> I, f: impl Fn(I) -> O) -> impl Fn(u64) -> Duration {
    fastrand::seed(7);
    let is: Vec<I> = (0..ARR_SIZE).map(|_| s()).collect();
    move |n| {
        let now = Instant::now();
        for i in 0..n {
            let _ = black_box(f(is[i as usize & (ARR_SIZE - 1)]));
        }
        now.elapsed()
    }
}
