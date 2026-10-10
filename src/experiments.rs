//! Candidate implementations under evaluation, enabled by the `experiments`
//! feature. Not part of the public API, and empty on the main branch.
//!
//! Add a candidate as a `#[inline] pub const fn` next to the function it would
//! replace, with a non-inlined wrapper in the `asm` module below, so that it
//! compiles the same way as the real function and its assembly can be dumped
//! with `scripts/asm_mca.sh`. Then add it to `benches/exp.rs` and to
//! `tests/exhaustive.rs`. See CONTRIBUTING.md for the workflow.

/// Non-inlined wrappers of the candidates for dumping their assembly.
#[cfg(feature = "asmdump")]
pub mod asm {}

const DAY_OFFSET: i32 = 3670 * 146097 + 719468;
const YEAR_OFFSET: i32 = 3670 * 400;

/// #104
#[inline]
pub const fn rd_to_date_table(n: i32) -> (i32, u8, u8) {
    let n = (n + DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let r = n % 146097;
    let n = r | 3;
    let p = 2939745 * n as u64;
    let z = (p / 2u64.pow(32)) as u32;
    let n = (p % 2u64.pow(32)) as u32 / 2939745 / 4;
    let j = n >= 306;
    let y = 100 * c + z + j as u32;
    let md = MONTH_DAY[n as usize];
    let y = (y as i32) - YEAR_OFFSET;
    (y, md as u8, (md >> 8) as u8)
}

const MONTH_DAY: [u16; 366] = month_day_table();

const fn month_day_table() -> [u16; 366] {
    let mut table = [0; 366];
    let mut n = 0;
    while n < 366 {
        let x = 2141 * n as u32 + 197913;
        let m = x / 2u32.pow(16);
        let d = x % 2u32.pow(16) / 2141;
        let m = if n >= 306 { m - 12 } else { m };
        table[n] = ((d + 1) << 8 | m) as u16;
        n += 1;
    }
    table
}

/// #106
#[inline]
pub const fn date_to_rd_table((y, m, d): (i32, u8, u8)) -> i32 {
    let y = (y + YEAR_OFFSET) as u32;
    let jf = (m < 3) as u32;
    let y = y - jf;
    let c = y / 100;
    let d = d as u32;
    let y = 1461 * y / 4 - c + c / 4;
    let m = MONTH_OFFSET[(m & 15) as usize];
    y as i32 + m + d as i32
}

const MONTH_OFFSET: [i32; 16] = month_offset_table();

const fn month_offset_table() -> [i32; 16] {
    let mut table = [0; 16];
    let mut m = 1;
    while m <= 12 {
        let mm = if m < 3 { m + 12 } else { m };
        table[m] = (979 * mm as i32 - 2919) / 32 - 1 - DAY_OFFSET;
        m += 1;
    }
    table
}

/// Hüffner 64-bit multiply-mask-compare on the year offset to be non-negative.
#[inline]
pub const fn is_leap_year_h64(y: i32) -> bool {
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
