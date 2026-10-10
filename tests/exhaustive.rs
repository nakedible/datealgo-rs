#![cfg(feature = "experiments")]
use datealgo::{RD_MAX, RD_MIN};

fn check(name: &str, f: fn(i32) -> (i32, u8, u8)) {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()) as i64;
    let total = RD_MAX as i64 - RD_MIN as i64 + 1;
    let chunk = (total + threads - 1) / threads;
    let bad: u64 = std::thread::scope(|s| {
        (0..threads)
            .map(|t| {
                s.spawn(move || {
                    let lo = RD_MIN as i64 + t * chunk;
                    let hi = (lo + chunk - 1).min(RD_MAX as i64);
                    let mut bad = 0u64;
                    for n in lo..=hi {
                        let n = n as i32;
                        if f(n) != datealgo::rd_to_date(n) {
                            if bad == 0 {
                                eprintln!("{name}: first mismatch at {n}: {:?} vs {:?}", f(n), datealgo::rd_to_date(n));
                            }
                            bad += 1;
                        }
                    }
                    bad
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum()
    });
    eprintln!("{name}: {bad} mismatches over {total} days");
    assert_eq!(bad, 0, "{name}");
}

#[test]
fn rd_to_date_j64() {
    check("j64", datealgo::experiments::rd_to_date_j64);
}

#[test]
fn rd_to_date_j32() {
    check("j32", datealgo::experiments::rd_to_date_j32);
}

#[test]
fn rd_to_date_jm() {
    check("jm", datealgo::experiments::rd_to_date_jm);
}

#[test]
fn rd_to_date_a() {
    check("a", datealgo::experiments::rd_to_date_a);
}

#[test]
fn rd_to_date_b() {
    check("b", datealgo::experiments::rd_to_date_b);
}

#[test]
fn rd_to_date_b2() {
    check("b2", datealgo::experiments::rd_to_date_b2);
}

#[test]
fn rd_to_date_c1() {
    check("c1", datealgo::experiments::rd_to_date_c1);
}

#[test]
fn rd_to_date_t() {
    check("t", datealgo::experiments::rd_to_date_t);
}

#[test]
fn rd_to_date_t2() {
    check("t2", datealgo::experiments::rd_to_date_t2);
}
