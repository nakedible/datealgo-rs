use std::time::{Duration, SystemTime, UNIX_EPOCH};

//pub const RATA_DIE_I16_MIN: i32 = gregorian_date_to_rata_die((i16::MIN, 1, 1));
pub const RATA_DIE_I16_MAX: i32 = gregorian_date_to_rata_die((i16::MAX as i32, 12, 31));
pub const UNIX_EPOCH_RATA_DIE: i32 = gregorian_date_to_rata_die((1970, 1, 1));
pub const UNIX_OFFSET: Duration = Duration::from_secs(UNIX_EPOCH_RATA_DIE as u64 * SECS_IN_DAY);

/// Selected to place unix epoch roughly in the center of the value space
const ERA_OFFSET: i32 = 3670;
const DAYS_IN_ERA: i32 = 146097;
const YEARS_IN_ERA: i32 = 400;
const UNIX_EPOCH_OFFSET: i32 = 719468;
const DAY_OFFSET: i32 = ERA_OFFSET * DAYS_IN_ERA + UNIX_EPOCH_OFFSET;
const YEAR_OFFSET: i32 = ERA_OFFSET * YEARS_IN_ERA;
const SECS_IN_DAY: u64 = 86400;

pub const RATA_DIE_MIN: i32 = gregorian_date_to_rata_die((-1467999, 1, 1));
pub const RATA_DIE_MAX: i32 = gregorian_date_to_rata_die((1471744, 12, 31));

// NOTES:
// - addition and substraction is the same speed regardless of signed or unsigned
// - addition and substraction is the same speed for u32 and u64
// - multiplication and especially division is slower for u64 than u32
// - division is slower for signed than unsigned
// - if the addition of two i32 is positive and fits in u32, wrapping (default)
//   semantics give us the correct results even if the sum is larger than i32::MAX

#[inline]
pub const fn rata_die_to_gregorian_date(n: i32) -> (i32, u32, u32) {
    let n = n.wrapping_add(DAY_OFFSET) as u32;
    let n = 4 * n + 3;
    let c = n / 146097;
    let n = n % 146097 / 4;
    let n = 4 * n + 3;
    let p = 2939745 * n as u64;
    let z = (p / 2u64.pow(32)) as u32;
    let n = (p % 2u64.pow(32)) as u32 / 2939745 / 4;
    let nd = (n >= 306) as u32;
    let y = 100 * c + z + nd;
    let n = 2141 * n + 197913;
    let m = n / 2u32.pow(16);
    let d = n % 2u32.pow(16) / 2141;
    let y = (y as i32).wrapping_sub(YEAR_OFFSET);
    let m = (m - 12 * nd);
    let d = (d + 1);
    (y, m, d)
}

#[inline]
pub const fn gregorian_date_to_rata_die((y, m, d): (i32, u32, u32)) -> i32 {
    let y = y.wrapping_add(YEAR_OFFSET) as u32;
    let jf = (m < 3) as u32;
    let y = y - jf;
    let m = m as u32 + 12 * jf;
    let d = d as u32 - 1;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = (979 * m - 2919) / 32;
    let n = y + m + d;
    (n as i32).wrapping_sub(DAY_OFFSET)
}

#[inline]
pub const fn secs_to_dhms(secs: u64) -> (u64, u8, u8, u8) {
    let (days, secs) = (secs / SECS_IN_DAY, (secs % SECS_IN_DAY) as u32);
    let ss = secs % 60;
    let mm = secs / 60 % 60;
    let hh = secs / 3600;
    (days, hh as u8, mm as u8, ss as u8)
}

#[inline]
pub const fn secs_to_dhms2(secs: u64) -> (u64, u8, u8, u8) {
    let ss = secs % 60;
    let secs = secs / 60;
    let mm = secs % 60;
    let secs = secs / 60;
    let hh = secs % 24;
    let secs = secs / 24;
    (secs, hh as u8, mm as u8, ss as u8)
}

#[inline]
pub fn from_systemtime(st: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
    let d = st.duration_since(UNIX_EPOCH).unwrap();
    let s = d.as_secs();
    let (days, hh, mm, ss) = secs_to_dhms(s);
    let days = days as i32 + UNIX_EPOCH_RATA_DIE;
    let (year, month, day) = rata_die_to_gregorian_date(days);
    let wday = (3 + days) % 7 + 1;
    (year as i16, month as u8, day as u8, hh, mm, ss, wday as u8)
}

#[inline]
pub fn to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
    let days = gregorian_date_to_rata_die((y as i32, m as u32, d as u32)) - UNIX_EPOCH_RATA_DIE;
    UNIX_EPOCH + Duration::from_secs(ss as u64 + mm as u64 * 60 + hh as u64 * 60 * 60 + (days as u64) * SECS_IN_DAY)
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    
    use super::*;

    #[test]
    fn test_consts() {
        //assert_eq!(RATA_DIE_I16_MIN, -12687794);
        assert_eq!(RATA_DIE_I16_MAX, 11248737);
        assert_eq!(UNIX_EPOCH_RATA_DIE, 0);
        assert_eq!(RATA_DIE_MIN, -536895152);
        assert_eq!(RATA_DIE_MAX, 536824295);
        assert_eq!(UNIX_OFFSET, Duration::from_secs(UNIX_EPOCH_RATA_DIE as u64 * SECS_IN_DAY));
    }

    #[test]
    fn test_date_to_rata() {
        assert_eq!(gregorian_date_to_rata_die((0, 3, 1)), -719468);
        assert_eq!(gregorian_date_to_rata_die((1970, 1, 1)), UNIX_EPOCH_RATA_DIE);
        //assert_eq!(gregorian_date_to_rata_die((i16::MIN, 1, 1)), RATA_DIE_I16_MIN);
        assert_eq!(gregorian_date_to_rata_die((i16::MAX as i32, 12, 31)), RATA_DIE_I16_MAX);

        // assert_eq!(
        //     format!("{:?}", strength_reduce::StrengthReducedU64::new(86400)),
        //     ""
        // );
    }

    #[test]
    fn b() {
        assert_eq!(rata_die_to_gregorian_date(0), (1970, 1, 1));
        assert_eq!(rata_die_to_gregorian_date(UNIX_EPOCH_RATA_DIE), (1970, 1, 1));
        //assert_eq!(rata_die_to_gregorian_date(RATA_DIE_I16_MIN), (i16::MIN, 1, 1));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_I16_MAX), (i16::MAX as i32, 12, 31));
        //assert_eq!(rata_die_to_gregorian_date(RATA_DIE_I16_MIN - 1), (i16::MAX, 12, 31));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_I16_MAX + 1), (i16::MAX as i32 + 1, 1, 1));
    }
}
