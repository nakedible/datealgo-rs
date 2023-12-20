use criterion::black_box;
use std::time::{Duration, Instant};

pub fn bencher<I: Copy, O>(s: impl Fn() -> I, f: impl Fn(I) -> O) -> impl Fn(u64) -> Duration {
    const ARR_SIZE: usize = 4096;
    move |n| {
        fastrand::seed(7);
        let is: [I; ARR_SIZE] = std::array::from_fn(|_| s());
        let now = Instant::now();
        for i in 0..n {
            let _ = black_box(f(is[i as usize & (ARR_SIZE - 1)]));
        }
        now.elapsed()
    }
}
