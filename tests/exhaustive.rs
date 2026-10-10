//! Exhaustive tests over the whole supported range of about a billion days.
//! They need a release build to finish in seconds, so they are ignored by
//! default:
//!
//! ```text
//! cargo test --release --test exhaustive -- --ignored
//! ```
//!
//! `check` also serves to compare a candidate from `datealgo::experiments`
//! against the current implementation, for example:
//!
//! ```text
//! check("candidate", RD_MIN, RD_MAX, |n| experiments::rd_to_date_x(n as i32) == rd_to_date(n as i32));
//! ```

// Tests need a newer compiler than the library MSRV.
#![allow(clippy::incompatible_msrv)]

use datealgo::*;

/// Check `ok` for every value from `lo` to `hi` inclusive on all CPU cores,
/// and fail with the first failing value and the number of failures.
fn check(name: &str, lo: i64, hi: i64, ok: impl Fn(i64) -> bool + Sync) {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()) as i64;
    let chunk = (hi - lo + threads) / threads;
    let ok = &ok;
    let results: Vec<(u64, Option<i64>)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                s.spawn(move || {
                    let start = lo + t * chunk;
                    let end = (start + chunk - 1).min(hi);
                    let mut failures = 0;
                    let mut first = None;
                    for n in start..=end {
                        if !ok(n) {
                            failures += 1;
                            first.get_or_insert(n);
                        }
                    }
                    (failures, first)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let failures: u64 = results.iter().map(|r| r.0).sum();
    let first = results.iter().find_map(|r| r.1);
    assert_eq!(failures, 0, "{name}: {failures} failures, first at {first:?}");
}

#[test]
#[ignore]
fn rd_to_date_round_trips() {
    check("rd_to_date", RD_MIN as i64, RD_MAX as i64, |n| {
        let (y, m, d) = rd_to_date(n as i32);
        (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m) && date_to_rd((y, m, d)) == n as i32
    });
}

#[test]
#[ignore]
fn weekdays_agree() {
    check("weekday", RD_MIN as i64, RD_MAX as i64, |n| {
        // 1970-01-01 was a Thursday
        let expected = ((n + 3).rem_euclid(7) + 1) as u8;
        rd_to_weekday(n as i32) == expected && date_to_weekday(rd_to_date(n as i32)) == expected
    });
}

#[test]
#[ignore]
fn adjacent_dates_agree() {
    check("adjacent dates", RD_MIN as i64, RD_MAX as i64 - 1, |n| {
        let n = n as i32;
        next_date(rd_to_date(n)) == rd_to_date(n + 1) && prev_date(rd_to_date(n + 1)) == rd_to_date(n)
    });
}

#[test]
#[ignore]
fn isoweekdates_round_trip() {
    check("isoweekdate", RD_MIN as i64, RD_MAX as i64, |n| {
        let n = n as i32;
        let iwd = rd_to_isoweekdate(n);
        isoweekdate_to_rd(iwd) == n && date_to_isoweekdate(rd_to_date(n)) == iwd
    });
}

fn check_date_to_rd(name: &str, f: fn((i32, u8, u8)) -> i32) {
    check(name, RD_MIN as i64, RD_MAX as i64, |n| f(rd_to_date(n as i32)) == n as i32);
}

#[test]
#[ignore]
fn date_to_rd_j() {
    check_date_to_rd("j", datealgo::experiments::date_to_rd_j);
}

#[test]
#[ignore]
fn date_to_rd_s() {
    check_date_to_rd("s", datealgo::experiments::date_to_rd_s);
}

#[test]
#[ignore]
fn date_to_rd_t() {
    check_date_to_rd("t", datealgo::experiments::date_to_rd_t);
}

#[test]
#[ignore]
fn date_to_rd_t2() {
    check_date_to_rd("t2", datealgo::experiments::date_to_rd_t2);
}

#[test]
#[ignore]
fn date_to_rd_t3() {
    check_date_to_rd("t3", datealgo::experiments::date_to_rd_t3);
}
