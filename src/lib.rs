use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const RATA_DIE_MIN: u32 = gregorian_date_to_rata_die((i16::MIN, 1, 1));
pub const RATA_DIE_MAX: u32 = gregorian_date_to_rata_die((i16::MAX, 12, 31));
pub const UNIX_EPOCH_RATA_DIE: u32 = gregorian_date_to_rata_die((1970, 1, 1));
pub const UNIX_OFFSET: Duration = Duration::from_secs(UNIX_EPOCH_RATA_DIE as u64 * SECS_IN_DAY);

const RATA_DIE_YEAR_OFFSET: i32 = 32800;
const SECS_IN_DAY: u64 = 86400;

pub const fn rata_die_to_gregorian_date(n: u32) -> (i16, u8, u8) {
    let n = 4 * n + 3;
    let c = n / 146097;
    let n = n % 146097 / 4;
    let n = 4 * n + 3;
    let p = 2939745 * n as u64;
    let z = (p / 2u64.pow(32)) as u32;
    let n = (p % 2u64.pow(32)) as u32 / 2939745 / 4;
    let nd = (n >= 306) as u32;
    let y = 100 * c + z;
    let n = 2141 * n + 197913;
    let m = n / 2u32.pow(16);
    let d = n % 2u32.pow(16) / 2141;
    let y = ((y + nd) as i32 - RATA_DIE_YEAR_OFFSET) as i16;
    let m = (m - 12 * nd) as u8;
    let d = (d + 1) as u8;
    (y, m, d)
}

pub const fn gregorian_date_to_rata_die((y, m, d): (i16, u8, u8)) -> u32 {
    let jf = (m < 3) as u32;
    let y = (y as i32 + RATA_DIE_YEAR_OFFSET) as u32 - jf;
    let m = m as u32 + 12 * jf;
    let d = d as u32 - 1;
    let c = y / 100;
    let y = 1461 * y / 4 - c + c / 4;
    let m = (979 * m - 2919) / 32;
    let n = y + m + d;
    n
}

pub const fn secs_to_dhms(secs: u64) -> (u64, u8, u8, u8) {
    let (days, secs) = (secs / SECS_IN_DAY, (secs % SECS_IN_DAY) as u32);
    let ss = secs % 60;
    let mm = secs / 60 % 60;
    let hh = secs / 3600;
    (days, hh as u8, mm as u8, ss as u8)
}

pub const fn secs_to_dhms2(secs: u64) -> (u64, u8, u8, u8) {
    let ss = secs % 60;
    let secs = secs / 60;
    let mm = secs % 60;
    let secs = secs / 60;
    let hh = secs % 24;
    let secs = secs / 24;
    (secs, hh as u8, mm as u8, ss as u8)
}

pub fn from_systemtime(st: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
    let d = st.duration_since(UNIX_EPOCH).unwrap();
    let s = d.as_secs();
    let (days, hh, mm, ss) = secs_to_dhms(s);
    let days = days as u32 + UNIX_EPOCH_RATA_DIE;
    let (year, month, day) = rata_die_to_gregorian_date(days as u32);
    let wday = (3 + days) % 7 + 1;
    (year, month, day, hh, mm, ss, wday as u8)
}

pub fn to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
    let days = gregorian_date_to_rata_die((y, m, d)) - UNIX_EPOCH_RATA_DIE;
    UNIX_EPOCH + Duration::from_secs(ss as u64 + mm as u64 * 60 + hh as u64 * 60 * 60 + (days as u64) * SECS_IN_DAY)
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    
    use super::*;

    #[test]
    fn test_consts() {
        assert_eq!(RATA_DIE_MIN, 11628);
        assert_eq!(RATA_DIE_MAX, 23948159);
        assert_eq!(UNIX_EPOCH_RATA_DIE, 12699422);
        assert_eq!(UNIX_OFFSET, Duration::from_secs(UNIX_EPOCH_RATA_DIE as u64 * SECS_IN_DAY));
    }

    #[test]
    fn a() {
        assert_eq!(gregorian_date_to_rata_die((0, 3, 1)), 11979954);
        assert_eq!(gregorian_date_to_rata_die((1970, 1, 1)), UNIX_EPOCH_RATA_DIE);
        assert_eq!(gregorian_date_to_rata_die((i16::MIN, 1, 1)), RATA_DIE_MIN);
        assert_eq!(gregorian_date_to_rata_die((i16::MAX, 12, 31)), RATA_DIE_MAX);

        // assert_eq!(
        //     format!("{:?}", strength_reduce::StrengthReducedU64::new(86400)),
        //     ""
        // );
    }

    #[test]
    fn b() {
        assert_eq!(rata_die_to_gregorian_date(11979954), (0, 3, 1));
        assert_eq!(rata_die_to_gregorian_date(UNIX_EPOCH_RATA_DIE), (1970, 1, 1));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_MIN), (i16::MIN, 1, 1));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_MAX), (i16::MAX, 12, 31));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_MIN - 1), (i16::MAX, 12, 31));
        assert_eq!(rata_die_to_gregorian_date(RATA_DIE_MAX + 1), (i16::MIN, 1, 1));
    }
}
