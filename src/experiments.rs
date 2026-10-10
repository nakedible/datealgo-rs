//! Candidate implementations under evaluation. Not part of the public API.
#![allow(missing_docs, clippy::all)]

use crate::{RD_MAX, RD_MIN};

/// Ben Joffe's 64-bit backward-counting algorithm.
#[inline]
pub const fn rd_to_date_j64(n: i32) -> (i32, u8, u8) {
    debug_assert!(n >= RD_MIN && n <= RD_MAX);
    const ERAS: i64 = 4_726_498_270;
    const D_SHIFT: i64 = 146_097 * ERAS - 719_469;
    const Y_SHIFT: i64 = 400 * ERAS - 1;
    const C1: u64 = 505_054_698_555_331;
    const C2: u64 = 50_504_432_782_230_121;
    const C3: u64 = 8_619_973_866_219_416;

    let rev = D_SHIFT - n as i64;
    let cen = (((rev as u64 as u128) * C1 as u128) >> 64) as i64;
    let jul = rev + cen - cen / 4;
    let num = (jul as u64 as u128) * C2 as u128;
    let yrs = Y_SHIFT - ((num >> 64) as i64);
    let ypt = ((782_432u128 * (num as u64) as u128) >> 64) as i64;
    let bump = ypt < 126_464;
    let shift = if bump { 191_360 } else { 977_792 };
    let n = (yrs.rem_euclid(4)) * 512 + shift - ypt;
    let d = (((((n as u64) & 0xFFFF) as u128) * C3 as u128) >> 64) as i64 + 1;
    let m = (n / 65_536) as u8;
    let y = yrs as i32 + bump as i32;
    (y, m, d as u8)
}

/// Ben Joffe's 32-bit fast variant (restricted range).
#[inline]
pub const fn rd_to_date_j32(n: i32) -> (i32, u8, u8) {
    const ERAS: u32 = 14_704;
    const D_SHIFT: u32 = 146_097 * ERAS - 719_469;
    const Y_SHIFT: u32 = 400 * ERAS - 1;
    const SCALE: u32 = 32;
    const SHIFT_0: u32 = 30_556 * SCALE;
    const SHIFT_1: u32 = 5_980 * SCALE;
    const C1: u64 = 505_054_698_555_331;
    const C2: u64 = 50_504_432_782_230_121;
    const C3: u64 = 8_619_973_866_219_416;

    let rev = D_SHIFT.wrapping_sub(n as u32);
    let cen = (((C1 as u128) * rev as u128) >> 64) as u32;
    let jul = rev.wrapping_sub(cen / 4).wrapping_add(cen);
    let num = (C2 as u128) * jul as u128;
    let yrs = Y_SHIFT.wrapping_sub((num >> 64) as u32);
    let low = num as u64;
    let ypt = ((((24_451 * SCALE) as u128) * low as u128) >> 64) as u32;
    let bump = ypt < (3_952 * SCALE);
    let phase = if bump { SHIFT_1 } else { SHIFT_0 };
    let n = (yrs % 4) * (16 * SCALE) + phase - ypt;
    let m = n / (2_048 * SCALE);
    let d = (((C3 as u128) * (n % (2_048 * SCALE)) as u128) >> 64) as u8 + 1;
    let y = yrs.wrapping_add(bump as u32) as i32;
    (y, m as u8, d)
}

/// Ben Joffe's 32-bit Julian map variant.
#[inline]
pub const fn rd_to_date_jm(n: i32) -> (i32, u8, u8) {
    let days = (n as u32).wrapping_add(0x8000_0000);
    let bckt = days >> 20;
    let bday = days - bckt * 1_022_679;
    let qday = bday * 4 + 524_943;
    let cent = qday / 146_097;
    let qjul = qday - (cent & !3) + cent * 4;
    let year = qjul / 1461;
    let yday = (qjul % 1461) / 4;
    let n = yday * 2141 + 197_913;
    let m = n / 65_536;
    let d = n % 65_536 / 2141;
    let bump = yday >= 306;
    let y = year as i32 + bckt as i32 * 2800 - 5_878_000 + bump as i32;
    let m = if bump { m - 12 } else { m };
    let d = d + 1;
    (y, m as u8, d as u8)
}

/// Julian map on the 32-bit range: Gregorian day count + (c - c/4) gives a
/// Julian day count, so the year is a single division by 1461 and the day of
/// year comes from the fraction of the same fixed-point multiply.
#[inline]
pub const fn rd_to_date_a(n: i32) -> (i32, u8, u8) {
    const K: u32 = 42;
    const M: u64 = (1u64 << K) / 1461 + 1;
    let nq = 4 * (n + crate::DAY_OFFSET) as u32 as u64 + 3;
    let c = nq / 146097;
    let j4 = nq + 4 * c - (c & !3);
    let p = j4 * M;
    let year = (p >> K) as u32;
    let doy = (((p & ((1 << K) - 1)) * 1461) >> (K + 2)) as u32;
    let n2 = 2141 * doy + 197913;
    let m = n2 >> 16;
    let d = (n2 & 0xffff) / 2141;
    let j = doy >= 306;
    let y = year as i32 - crate::YEAR_OFFSET + j as i32;
    let m = if j { m - 12 } else { m };
    (y, m as u8, d as u8 + 1)
}

/// Like `a`, but in 32 bits where possible and with a 2^64 fixed-point
/// inverse of 1461 so the year and the fraction come from one widening multiply.
#[inline]
pub const fn rd_to_date_b(n: i32) -> (i32, u8, u8) {
    const M: u64 = (u64::MAX / 1461) + 1;
    let nq = 4 * (n + crate::DAY_OFFSET) as u32 + 3;
    let c = nq / 146097;
    let j4 = nq + 4 * c - (c & !3);
    let p = j4 as u128 * M as u128;
    let year = (p >> 64) as u32;
    let frac = p as u64;
    let doy = (((frac >> 32) * 1461) >> 34) as u32;
    let n2 = 2141 * doy + 197913;
    let m = n2 >> 16;
    let d = (n2 & 0xffff) / 2141;
    let j = doy >= 306;
    let y = year as i32 - crate::YEAR_OFFSET + j as i32;
    let m = if j { m - 12 } else { m };
    (y, m as u8, d as u8 + 1)
}

/// Like `b`, but the day of year from the full 64-bit fraction.
#[inline]
pub const fn rd_to_date_b2(n: i32) -> (i32, u8, u8) {
    const M: u64 = (u64::MAX / 1461) + 1;
    let nq = 4 * (n + crate::DAY_OFFSET) as u32 + 3;
    let c = nq / 146097;
    let j4 = nq + 4 * c - (c & !3);
    let p = j4 as u128 * M as u128;
    let year = (p >> 64) as u32;
    let frac = p as u64;
    let doy = ((frac as u128 * 1461) >> 66) as u32;
    let n2 = 2141 * doy + 197913;
    let m = n2 >> 16;
    let d = (n2 & 0xffff) / 2141;
    let j = doy >= 306;
    let y = year as i32 - crate::YEAR_OFFSET + j as i32;
    let m = if j { m - 12 } else { m };
    (y, m as u8, d as u8 + 1)
}

/// Current algorithm, reshaped: the month offset is chosen before the
/// multiply-add, and the year terms are ordered so the additions can merge.
#[inline]
pub const fn rd_to_date_c1(n: i32) -> (i32, u8, u8) {
    let n = (n + crate::DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let r = n % 146097;
    let n = r | 3;
    let p = 2939745 * n as u64;
    let z = (p >> 32) as u32;
    let n = (p as u32) / 2939745 / 4;
    let j = n >= 306;
    let y = (100 * c + z) as i32 + (j as i32 - crate::YEAR_OFFSET);
    let shift = if j { 197913u32.wrapping_sub(12 << 16) } else { 197913 };
    let n = (2141 * n).wrapping_add(shift);
    let m = n >> 16;
    let d = (n & 0xffff) / 2141;
    (y, m as u8, d as u8 + 1)
}

const fn md_table() -> [u16; 366] {
    let mut t = [0u16; 366];
    let mut doy = 0;
    while doy < 366 {
        let n = 2141 * doy as u32 + 197913;
        let m = n >> 16;
        let d = (n & 0xffff) / 2141 + 1;
        let m = if doy >= 306 { m - 12 } else { m };
        t[doy] = ((d << 8) | m) as u16;
        doy += 1;
    }
    t
}
const MD: [u16; 366] = md_table();

/// Current algorithm, with month and day looked up from the day of year.
#[inline]
pub const fn rd_to_date_t(n: i32) -> (i32, u8, u8) {
    let n = (n + crate::DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let r = n % 146097;
    let n = r | 3;
    let p = 2939745 * n as u64;
    let z = (p >> 32) as u32;
    let n = (p as u32) / 2939745 / 4;
    let j = n >= 306;
    let y = (100 * c + z) as i32 + (j as i32 - crate::YEAR_OFFSET);
    let md = MD[n as usize];
    (y, md as u8, (md >> 8) as u8)
}

const fn month_table() -> [u32; 15] {
    // Indexed by the March-based month index 3..14 from Neri's formula:
    // the day of year the month starts on (minus one, so day = doy - start)
    // in the low 16 bits and the calendar month in the high 16 bits.
    let mut t = [0u32; 15];
    let mut doy = 0u32;
    while doy < 366 {
        let n = 2141 * doy + 197913;
        let m = n >> 16;
        let d = (n & 0xffff) / 2141;
        if d == 0 {
            let cm = if m > 12 { m - 12 } else { m };
            t[m as usize] = (cm << 16) | (doy.wrapping_sub(1) & 0xffff);
        }
        doy += 1;
    }
    t
}
const MONTHS: [u32; 15] = month_table();

/// Current algorithm, with the day from a 13-entry month start table instead of
/// a second multiply.
#[inline]
pub const fn rd_to_date_t2(n: i32) -> (i32, u8, u8) {
    let n = (n + crate::DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let r = n % 146097;
    let n = r | 3;
    let p = 2939745 * n as u64;
    let z = (p >> 32) as u32;
    let n = (p as u32) / 2939745 / 4;
    let j = n >= 306;
    let y = (100 * c + z) as i32 + (j as i32 - crate::YEAR_OFFSET);
    let m = (2141 * n + 197913) >> 16;
    let e = MONTHS[m as usize];
    let d = n.wrapping_sub(e) as u8;
    (y, (e >> 16) as u8, d)
}

#[cfg(feature = "asmdump")]
pub mod asm {
    #[inline(never)]
    pub const fn rd_to_date_j64(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_j64(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_j32(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_j32(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_jm(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_jm(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_a(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_a(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_b(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_b(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_b2(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_b2(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_c1(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_c1(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_t(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_t(n)
    }
    #[inline(never)]
    pub const fn rd_to_date_t2(n: i32) -> (i32, u8, u8) {
        super::rd_to_date_t2(n)
    }
}
