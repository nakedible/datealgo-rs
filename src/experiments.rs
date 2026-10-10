//! Candidate implementations under evaluation, enabled by the `experiments`
//! feature. Not part of the public API, and empty on the main branch.
//!
//! Add a candidate as a `#[inline] pub const fn` next to the function it would
//! replace, with a non-inlined wrapper in the `asm` module below, so that it
//! compiles the same way as the real function and its assembly can be dumped
//! with `scripts/asm_mca.sh`. Then add it to `benches/exp.rs` and to
//! `tests/exhaustive.rs`. See CONTRIBUTING.md for the workflow.

use crate::{DAY_OFFSET, YEAR_OFFSET};

/// Ben Joffe's inverse: original month with a per-range constant, and
/// `365 * y + y / 4` instead of `1461 * y / 4`.
#[inline]
pub const fn date_to_rd_j((y, m, d): (i32, u8, u8)) -> i32 {
    let bump = m <= 2;
    let yrs = (y + YEAR_OFFSET) as u32 - bump as u32;
    let cen = yrs / 100;
    let shift: i32 = if bump { 8829 } else { -2919 };
    let year_days = yrs * 365 + yrs / 4 - cen + cen / 4;
    let month_days = (979 * m as i32 + shift) as u32 / 32;
    (year_days + month_days + d as u32 - 1) as i32 - DAY_OFFSET
}

/// Current algorithm with Joffe's per-range month constant.
#[inline]
pub const fn date_to_rd_s((y, m, d): (i32, u8, u8)) -> i32 {
    let jf = m < 3;
    let y = (y + YEAR_OFFSET) as u32 - jf as u32;
    let c = y / 100;
    let shift: i32 = if jf { 8829 } else { -2919 };
    let y = 1461 * y / 4 - c + c / 4;
    let m = (979 * m as i32 + shift) as u32 / 32;
    (y + m + d as u32 - 1) as i32 - DAY_OFFSET
}

const fn month_days_table() -> [u16; 13] {
    let mut t = [0u16; 13];
    let mut m = 1;
    while m <= 12 {
        let mm = if m < 3 { m + 12 } else { m };
        t[m as usize] = ((979 * mm - 2919) / 32) as u16;
        m += 1;
    }
    t
}
const MONTH_DAYS: [u16; 13] = month_days_table();

/// Current algorithm with the days before the month (counted from March 1st)
/// looked up from a 13-entry table.
#[inline]
pub const fn date_to_rd_t((y, m, d): (i32, u8, u8)) -> i32 {
    let jf = m < 3;
    let y = (y + YEAR_OFFSET) as u32 - jf as u32;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = MONTH_DAYS[m as usize] as u32;
    (y + m + d as u32 - 1) as i32 - DAY_OFFSET
}

const fn month_days_table16() -> [u16; 16] {
    let mut t = [0u16; 16];
    let mut m = 1;
    while m <= 12 {
        let mm = if m < 3 { m + 12 } else { m };
        t[m as usize] = ((979 * mm - 2919) / 32) as u16;
        m += 1;
    }
    t
}
const MONTH_DAYS16: [u16; 16] = month_days_table16();

/// Like `t`, but a 16-entry table indexed by `m & 15`, so there is no bounds
/// check and no panic for invalid months.
#[inline]
pub const fn date_to_rd_t2((y, m, d): (i32, u8, u8)) -> i32 {
    let jf = m < 3;
    let y = (y + YEAR_OFFSET) as u32 - jf as u32;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = MONTH_DAYS16[(m & 15) as usize] as u32;
    (y + m + d as u32 - 1) as i32 - DAY_OFFSET
}

const fn month_offset_table() -> [u32; 16] {
    let mut t = [0u32; 16];
    let mut m = 1;
    while m <= 12 {
        let mm = if m < 3 { m + 12 } else { m };
        t[m as usize] = ((979 * mm - 2919) / 32u32).wrapping_sub(1).wrapping_sub(DAY_OFFSET as u32);
        m += 1;
    }
    t
}
const MONTH_OFFSET: [u32; 16] = month_offset_table();

/// Like `t2`, with `- 1 - DAY_OFFSET` folded into the table entries.
#[inline]
pub const fn date_to_rd_t3((y, m, d): (i32, u8, u8)) -> i32 {
    let jf = m < 3;
    let y = (y + YEAR_OFFSET) as u32 - jf as u32;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = MONTH_OFFSET[(m & 15) as usize];
    y.wrapping_add(m).wrapping_add(d as u32) as i32
}

/// Joffe's per-range month constant, with `- 1 - DAY_OFFSET` folded into the
/// month term by doing its division in 64 bits. No table.
#[inline]
pub const fn date_to_rd_s3((y, m, d): (i32, u8, u8)) -> i32 {
    const K: u64 = 32 * ((-1 - DAY_OFFSET) as u32 as u64);
    let jf = m < 3;
    let y = (y + YEAR_OFFSET) as u32 - jf as u32;
    let c = y / 100;
    let shift: u64 = if jf { 8829 + K } else { K - 2919 };
    let y = 1461 * y / 4 - c + c / 4;
    let m = ((979 * m as u64 + shift) >> 5) as u32;
    y.wrapping_add(m).wrapping_add(d as u32) as i32
}

/// Non-inlined wrappers of the candidates for dumping their assembly.
#[cfg(feature = "asmdump")]
pub mod asm {
    #[inline(never)]
    pub const fn date_to_rd_j(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_j(d)
    }
    #[inline(never)]
    pub const fn date_to_rd_s(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_s(d)
    }
    #[inline(never)]
    pub const fn date_to_rd_t(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_t(d)
    }
    #[inline(never)]
    pub const fn date_to_rd_t2(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_t2(d)
    }
    #[inline(never)]
    pub const fn date_to_rd_t3(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_t3(d)
    }
    #[inline(never)]
    pub const fn date_to_rd_s3(d: (i32, u8, u8)) -> i32 {
        super::date_to_rd_s3(d)
    }
}
