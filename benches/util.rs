use criterion::black_box;
use std::time::{Duration, Instant};

pub fn bencher<I: Copy, O>(s: impl Fn() -> I, f: impl Fn(I) -> O) -> impl Fn(u64) -> Duration {
    move |n| {
        let v = s();
        let now = Instant::now();
        for _ in 0..n {
            let _ = black_box(f(v));
        }
        now.elapsed()
    }
}
