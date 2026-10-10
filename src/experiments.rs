//! Candidate implementations under evaluation, enabled by the `experiments`
//! feature. Not part of the public API, and empty on the main branch.
//!
//! Add a candidate as a `#[inline] pub const fn` next to the function it would
//! replace, with a non-inlined wrapper in the `asm` module below, so that it
//! compiles the same way as the real function and its assembly can be dumped
//! with `scripts/asm_mca.sh`. Then add it to `benches/exp.rs` and to
//! `tests/exhaustive.rs`. See CONTRIBUTING.md for the workflow.

use crate::{consts, YEAR_MAX, YEAR_MIN};

const YEAR_OFFSET: i32 = 3670 * 400;

/// Hüffner 64-bit multiply-mask-compare on the year offset to be non-negative.
#[inline]
pub const fn is_leap_year_h64(y: i32) -> bool {
    debug_assert!(y >= YEAR_MIN && y <= YEAR_MAX, "given year is out of range");
    const F: u64 = 0x4000_0000_28F5_C28F;
    const M: u64 = 0xC000_000F_8000_000F;
    const T: u64 = 0xF_8000_0000;
    let y = (y + YEAR_OFFSET) as u32 as u64;
    y.wrapping_mul(F) & M <= T
}

/// Hüffner 64-bit on the absolute value, as in time-core.
#[inline]
pub const fn is_leap_year_abs(y: i32) -> bool {
    (y as i64).unsigned_abs().wrapping_mul(0x4000_0000_28F5_C28F) & 0xC000_000F_8000_000F <= 0xF_8000_0000
}

/// Neri-Schneider `% 25` on the offset unsigned year.
#[inline]
pub const fn is_leap_year_u25(y: i32) -> bool {
    let y = (y + YEAR_OFFSET) as u32;
    if y % 25 != 0 {
        y & 3 == 0
    } else {
        y & 15 == 0
    }
}

/// Current structure with the Hüffner leap year.
#[inline]
pub const fn days_in_month_h64(y: i32, m: u8) -> u8 {
    debug_assert!(m >= consts::MONTH_MIN && m <= consts::MONTH_MAX, "given month is out of range");
    if m != 2 {
        30 | (m ^ (m >> 3))
    } else if is_leap_year_h64(y) {
        29
    } else {
        28
    }
}

/// Branchless: the Neri-Schneider month length, corrected for February.
#[inline]
pub const fn days_in_month_bl(y: i32, m: u8) -> u8 {
    let leap = is_leap_year_h64(y) as u8;
    (30 | (m ^ (m >> 3))) - (((m == 2) as u8) * (2 - leap))
}

/// Branchless: two-bit lengths packed into a constant, selected by leap year.
#[inline]
pub const fn days_in_month_bits(y: i32, m: u8) -> u8 {
    const C: u32 = bits(0);
    const CL: u32 = bits(1);
    const fn bits(leap: u32) -> u32 {
        let lens = [0, 3, leap, 3, 2, 3, 2, 3, 3, 2, 3, 2, 3];
        let mut c = 0;
        let mut i = 1;
        while i <= 12 {
            c |= lens[i] << (2 * i);
            i += 1;
        }
        c
    }
    let c = if is_leap_year_h64(y) { CL } else { C };
    28 + ((c >> (2 * (m as u32 & 15))) & 3) as u8
}

/// Current structure with the Hüffner leap year.
#[inline]
pub const fn next_date_h64((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
    if d < 28 || d < days_in_month_h64(y, m) {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

/// Branchless carry from day to month to year.
#[inline]
pub const fn next_date_bl((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
    let last_d = d >= days_in_month_bl(y, m);
    let last_m = last_d & (m == 12);
    let d = if last_d { 1 } else { d + 1 };
    let m = if last_m { 1 } else { m + last_d as u8 };
    (y + last_m as i32, m, d)
}

/// Fast path for days below 28, branchless carry otherwise.
#[inline]
pub const fn next_date_fast((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
    if d < 28 {
        return (y, m, d + 1);
    }
    next_date_bl((y, m, d))
}

/// Current structure with the Hüffner leap year.
#[inline]
pub const fn prev_date_h64((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
    if d > 1 {
        (y, m, d - 1)
    } else if m > 1 {
        (y, m - 1, days_in_month_h64(y, m - 1))
    } else {
        (y - 1, 12, 31)
    }
}

/// Branchless borrow from day to month to year.
#[inline]
pub const fn prev_date_bl((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
    let first_d = d == 1;
    let first_m = first_d & (m == 1);
    let y = y - first_m as i32;
    let pm = if m == 1 { 12 } else { m - 1 };
    let dim = days_in_month_bl(y, pm);
    let d = if first_d { dim } else { d - 1 };
    let m = if first_d { pm } else { m };
    (y, m, d)
}

/// Non-inlined wrappers of the candidates for dumping their assembly.
#[cfg(feature = "asmdump")]
pub mod asm {
    macro_rules! wrap {
        ($($name:ident($($a:ident: $t:ty),*) -> $r:ty;)*) => {
            $(#[inline(never)]
            pub const fn $name($($a: $t),*) -> $r {
                super::$name($($a),*)
            })*
        };
    }
    wrap! {
        is_leap_year_h64(y: i32) -> bool;
        is_leap_year_abs(y: i32) -> bool;
        is_leap_year_u25(y: i32) -> bool;
        days_in_month_h64(y: i32, m: u8) -> u8;
        days_in_month_bl(y: i32, m: u8) -> u8;
        days_in_month_bits(y: i32, m: u8) -> u8;
        next_date_h64(x: (i32, u8, u8)) -> (i32, u8, u8);
        next_date_bl(x: (i32, u8, u8)) -> (i32, u8, u8);
        next_date_fast(x: (i32, u8, u8)) -> (i32, u8, u8);
        prev_date_h64(x: (i32, u8, u8)) -> (i32, u8, u8);
        prev_date_bl(x: (i32, u8, u8)) -> (i32, u8, u8);
    }
}
