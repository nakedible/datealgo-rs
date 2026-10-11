//! Inputs of the basic benchmarks, shared by `benches/basic_zen.rs` and
//! `benches/basic_ab.rs`.

#![allow(dead_code)]

use std::{sync::Arc, time::SystemTime};

// Large enough that the branch predictor can't learn the input sequence, as
// in the Criterion helper in util.rs. A power of two, so inputs can be indexed
// with a mask. Indexing with `%` by a runtime length costs a division per
// iteration, which hides the cheapest functions behind it.
pub const INPUT_SET_SIZE: usize = 262144;
const _: () = assert!(INPUT_SET_SIZE.is_power_of_two());

pub fn seeded_inputs<I>(mut generate: impl FnMut() -> I) -> Arc<[I]> {
    fastrand::seed(7);
    (0..INPUT_SET_SIZE).map(|_| generate()).collect::<Vec<_>>().into()
}

pub fn rand_year() -> i32 {
    fastrand::i32(datealgo::YEAR_MIN..=datealgo::YEAR_MAX)
}

pub fn rand_rd() -> i32 {
    fastrand::i32(datealgo::RD_MIN..=datealgo::RD_MAX)
}

pub fn rand_date() -> (i32, u8, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    let d = fastrand::u8(1..=datealgo::days_in_month(y, m));
    (y, m, d)
}

pub fn rand_secs() -> i64 {
    fastrand::i64(datealgo::RD_SECONDS_MIN..=datealgo::RD_SECONDS_MAX)
}

pub fn rand_hms() -> (u8, u8, u8) {
    let h = fastrand::u8(0..=23);
    let m = fastrand::u8(0..=59);
    let s = fastrand::u8(0..=59);
    (h, m, s)
}

pub fn rand_dhms() -> (i32, u8, u8, u8) {
    let rd = rand_rd();
    let (h, m, s) = rand_hms();
    (rd, h, m, s)
}

pub fn rand_dt() -> (i32, u8, u8, u8, u8, u8) {
    let (y, m, d) = rand_date();
    let (hh, mm, ss) = rand_hms();
    (y, m, d, hh, mm, ss)
}

pub fn rand_ym() -> (i32, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    (y, m)
}

pub fn rand_sn() -> (i64, u32) {
    let s = rand_secs();
    let n = fastrand::u32(0..=999_999_999);
    (s, n)
}

pub fn rand_st() -> SystemTime {
    datealgo::secs_to_systemtime(rand_sn()).unwrap()
}

pub fn rand_dtn() -> (i32, u8, u8, u8, u8, u8, u32) {
    let (y, m, d, hh, mm, ss) = rand_dt();
    let n = fastrand::u32(0..=999_999_999);
    (y, m, d, hh, mm, ss, n)
}

pub fn rand_iwd() -> (i32, u8, u8) {
    datealgo::rd_to_isoweekdate(rand_rd())
}
