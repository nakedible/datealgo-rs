use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant, SystemTime};

#[cfg(target_arch = "x86")]
use std::arch::x86;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64 as x86;

#[inline]
fn start_tsc() -> u64 {
    unsafe { x86::_mm_lfence() };
    let tsc = unsafe { x86::_rdtsc() };
    unsafe { x86::_mm_lfence() };
    tsc
}

fn stop_tsc() -> u64 {
    let tsc = unsafe { x86::__rdtscp(&mut 0) };
    unsafe { x86::_mm_lfence() };
    tsc
}

fn bencher<I: Copy, O>(s: impl Fn() -> I, f: impl Fn(I) -> O) -> impl Fn(u64) -> Duration {
    move |n| {
        let v = s();
        //let now = Instant::now();
        let start = start_tsc();
        for _ in 0..n {
            let _ = black_box(f(v));
        }
        let end = stop_tsc();
        let diff = end.saturating_sub(start);
        Duration::from_nanos(diff)
        //now.elapsed()
    }
}

fn rand_year() -> i32 {
    fastrand::i32(1970..=9999)
}

fn rand_rd() -> i32 {
    fastrand::i32(datealgo::date_to_rd((1970, 1, 1))..=datealgo::date_to_rd((9999, 1, 1)))
}

fn rand_date() -> (i32, u8, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    let d = fastrand::u8(1..=datealgo::days_in_month(y, m));
    (y, m, d)
}

fn rand_secs() -> i64 {
    fastrand::i64(datealgo::datetime_to_secs((1970, 1, 1, 0, 0, 0))..=datealgo::datetime_to_secs((9999, 12, 31, 23, 59, 59)))
}

fn rand_hms() -> (u8, u8, u8) {
    let h = fastrand::u8(0..=23);
    let m = fastrand::u8(0..=59);
    let s = fastrand::u8(0..59);
    (h, m, s)
}

fn rand_dhms() -> (i32, u8, u8, u8) {
    let rd = rand_rd();
    let (h, m, s) = rand_hms();
    (rd, h, m, s)
}

fn rand_dt() -> (i32, u8, u8, u8, u8, u8) {
    let (y, m, d) = rand_date();
    let (hh, mm, ss) = rand_hms();
    (y, m, d, hh, mm, ss)
}

fn rand_ym() -> (i32, u8) {
    let y = rand_year();
    let m = fastrand::u8(1..=12);
    (y, m)
}

fn rand_sn() -> (i64, u32) {
    let s = rand_secs();
    let n = fastrand::u32(0..=999_999_999);
    (s, n)
}

fn rand_st() -> SystemTime {
    datealgo::secs_to_systemtime(rand_sn()).unwrap()
}

fn rand_dtn() -> (i32, u8, u8, u8, u8, u8, u32) {
    let (y, m, d, hh, mm, ss) = rand_dt();
    let n = fastrand::u32(0..=999_999_999);
    (y, m, d, hh, mm, ss, n)
}

fn rand_iwd() -> (i32, u8, u8) {
    datealgo::rd_to_isoweekdate(rand_rd())
}

mod datealgo_alt {
    const YEAR_OFFSET: i32 = 3670 * 400;
    const DAY_OFFSET: i32 = 3670 * 146097 + 719468;
    const SECS_OFFSET: i64 = DAY_OFFSET as i64 * 86400;
    const SECS_IN_DAY: i64 = 86400;

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub const fn secs_to_dhms(secs: i64) -> (i32, u8, u8, u8) {
        let secs = secs as u64;
        let ss = secs % 60;
        let secs = secs / 60;
        let mm = secs % 60;
        let secs = secs / 60;
        let hh = secs % 24;
        let secs = secs / 24;
        (secs as i32, hh as u8, mm as u8, ss as u8)
    }

    #[inline]
    pub const fn secs_to_dhms2(secs: i64) -> (i32, u8, u8, u8) {
        let secs = if secs > datealgo::RD_SECONDS_MAX { 0 } else { secs }; // allows compiler to optimize more
        let secs = secs.wrapping_add(SECS_OFFSET) as u64;
        let days = (secs / SECS_IN_DAY as u64) as u32;
        let secs = (secs % SECS_IN_DAY as u64) as u32;
        let ss = secs % 60;
        let mm = secs / 60 % 60;
        let hh = secs / 3600;
        let days = (days as i32).wrapping_sub(DAY_OFFSET);
        (days, hh as u8, mm as u8, ss as u8)
    }

    #[inline]
    pub const fn rd_to_weekday(n: i32) -> u8 {
        (n + 4).rem_euclid(7) as u8
    }

    #[inline]
    pub const fn rd_to_weekday2(n: i32) -> u8 {
        if n >= -4 {
            ((n + 4) % 7) as u8
        } else {
            ((n + 5) % 7 + 6) as u8
        }
    }

    #[inline]
    pub const fn date_to_weekday((y, m, d): (i32, u8, u8)) -> u8 {
        let y = y.wrapping_add(YEAR_OFFSET) as u32 - (m < 3) as u32;
        let t = [6u8, 2, 1, 4, 6, 2, 4, 0, 3, 5, 1, 3];
        let mut idx = m.wrapping_sub(1) as usize;
        if idx > 11 {
            idx = 0;
        } // ensure no bounds check
        ((y + y / 4 - y / 100 + y / 400 + t[idx] as u32 + d as u32) % 7 + 1) as u8
    }

    #[inline]
    pub const fn date_to_weekday2((year, month, day): (i32, u8, u8)) -> u8 {
        let year = year.wrapping_add(YEAR_OFFSET) as u32;
        let adjustment = (14 - month) / 12;
        let mm = (month + 12 * adjustment - 2) as u32;
        let yy = year - adjustment as u32;
        ((day as u32 + (13 * mm - 1) / 5 + yy + yy / 4 - yy / 100 + yy / 400 + 6) % 7 + 1) as u8
    }

    #[inline]
    pub const fn next_date((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
        let rd = datealgo::date_to_rd((y, m, d));
        datealgo::rd_to_date(rd + 1)
    }

    #[inline]
    pub const fn prev_date((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
        let rd = datealgo::date_to_rd((y, m, d));
        datealgo::rd_to_date(rd - 1)
    }

    #[inline]
    pub const fn is_leap_year(y: i32) -> bool {
        y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
    }

    #[inline]
    pub const fn is_leap_year2(y: i32) -> bool {
        let y = y.wrapping_add(YEAR_OFFSET) as u32;
        y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
    }

    #[inline]
    pub const fn is_leap_year3(y: i32) -> bool {
        if y % 100 != 0 {
            y % 4 == 0
        } else {
            y % 16 == 0
        }
    }

    #[inline]
    pub const fn is_leap_year4(y: i32) -> bool {
        let y = y.wrapping_add(YEAR_OFFSET) as u32;
        if y % 100 != 0 {
            y % 4 == 0
        } else {
            y % 16 == 0
        }
    }

    #[inline]
    pub const fn days_in_month(y: i32, m: u8) -> u8 {
        // ensure compiler doesn't include a bounds check
        if m >= datealgo::consts::MONTH_MIN && m <= datealgo::consts::MONTH_MAX {
            let idx = m as usize - 1;
            if datealgo::is_leap_year(y) {
                [31u8, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx]
            } else {
                [31u8, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][idx]
            }
        } else {
            0
        }
    }

    #[inline]
    pub const fn days_in_month2(y: i32, m: u8) -> u8 {
        if m == 2 {
            if datealgo::is_leap_year(y) {
                29
            } else {
                28
            }
        } else {
            30 | (9 * m / 8)
        }
    }

    #[cfg(feature = "std")]
    const SECS_OFFSET_DURATION: Duration = Duration::from_secs(SECS_OFFSET as u64);

    #[inline]
    pub fn systemtime_to_secs(st: SystemTime) -> Option<(i64, u32)> {
        let dur = st.duration_since(UNIX_EPOCH - SECS_OFFSET_DURATION).ok()?;
        let secs = dur.as_secs();
        // if secs < (SECS_OFFSET + SECS_MIN) as u64 || secs > (SECS_OFFSET + SECS_MAX) as u64 {
        //     return None;
        // }
        let nsecs = dur.subsec_nanos();
        Some((secs as i64 - SECS_OFFSET, nsecs))
    }
}

mod httpdate {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u8, u8) {
        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365 * 400 + 97;
        const DAYS_PER_100Y: i64 = 365 * 100 + 24;
        const DAYS_PER_4Y: i64 = 365 * 4 + 1;

        let days = n as i64 - LEAPOCH;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays + 1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        (year as i32, mon as u8, mday as u8)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u8, u8)) -> i32 {
        fn is_leap_year(y: u16) -> bool {
            y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
        }

        let y = y as u16;
        let leap_years = ((y - 1) - 1968) / 4 - ((y - 1) - 1900) / 100 + ((y - 1) - 1600) / 400;
        let mut ydays = match m {
            1 => 0,
            2 => 31,
            3 => 59,
            4 => 90,
            5 => 120,
            6 => 151,
            7 => 181,
            8 => 212,
            9 => 243,
            10 => 273,
            11 => 304,
            12 => 334,
            _ => unreachable!(),
        } + d as u64
            - 1;
        if is_leap_year(y) && m > 2 {
            ydays += 1;
        }
        let days = (y as u64 - 1970) * 365 + leap_years as u64 + ydays;
        days as i32
    }

    pub fn systemtime_to_datetime(v: SystemTime) -> (i32, u8, u8, u8, u8, u8, u32) {
        let dur = v.duration_since(UNIX_EPOCH).expect("all times should be after the epoch");
        let secs_since_epoch = dur.as_secs();

        if secs_since_epoch >= 253402300800 {
            // year 9999
            panic!("date must be before year 9999");
        }

        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365 * 400 + 97;
        const DAYS_PER_100Y: i64 = 365 * 100 + 24;
        const DAYS_PER_4Y: i64 = 365 * 4 + 1;

        let days = (secs_since_epoch / 86400) as i64 - LEAPOCH;
        let secs_of_day = secs_since_epoch % 86400;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays + 1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        // let mut wday = (3 + days) % 7;
        // if wday <= 0 {
        //     wday += 7
        // };
        (
            year as i32,
            mon as u8,
            mday as u8,
            (secs_of_day / 3600) as u8,
            ((secs_of_day % 3600) / 60) as u8,
            (secs_of_day % 60) as u8,
            dur.subsec_nanos(),
        )
    }

    pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u8, u8, u8, u8, u8, u32)) -> SystemTime {
        fn is_leap_year(y: i32) -> bool {
            y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
        }

        let leap_years = ((y - 1) - 1968) / 4 - ((y - 1) - 1900) / 100 + ((y - 1) - 1600) / 400;
        let mut ydays = match m {
            1 => 0,
            2 => 31,
            3 => 59,
            4 => 90,
            5 => 120,
            6 => 151,
            7 => 181,
            8 => 212,
            9 => 243,
            10 => 273,
            11 => 304,
            12 => 334,
            _ => unreachable!(),
        } + d as u64
            - 1;
        if is_leap_year(y) && m > 2 {
            ydays += 1;
        }
        let days = (y as u64 - 1970) * 365 + leap_years as u64 + ydays;
        UNIX_EPOCH + Duration::new(ss as u64 + mm as u64 * 60 + hh as u64 * 3600 + days * 86400, nsec)
    }
}

mod humantime {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u8, u8) {
        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365 * 400 + 97;
        const DAYS_PER_100Y: i64 = 365 * 100 + 24;
        const DAYS_PER_4Y: i64 = 365 * 4 + 1;

        let days = n as i64 - LEAPOCH;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays + 1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        (year as i32, mon as u8, mday as u8)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u8, u8)) -> i32 {
        fn is_leap_year(y: u64) -> bool {
            y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
        }
        let year = y as u64;
        let month = m as u64;
        let day = d as u64;
        let leap = is_leap_year(year);
        let (mut ydays, _mdays) = match month {
            1 => (0, 31),
            2 if leap => (31, 29),
            2 => (31, 28),
            3 => (59, 31),
            4 => (90, 30),
            5 => (120, 31),
            6 => (151, 30),
            7 => (181, 31),
            8 => (212, 31),
            9 => (243, 30),
            10 => (273, 31),
            11 => (304, 30),
            12 => (334, 31),
            _ => panic!(),
        };
        ydays += day - 1;
        if leap && month > 2 {
            ydays += 1;
        }

        let leap_years = ((year - 1) - 1968) / 4 - ((year - 1) - 1900) / 100 + ((year - 1) - 1600) / 400;
        let days = (year - 1970) * 365 + leap_years + ydays;
        days as i32
    }

    pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u8, u8, u8, u8, u8, u32)) -> SystemTime {
        fn is_leap_year(y: u64) -> bool {
            y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
        }
        let year = y as u64;
        let month = m as u64;
        let day = d as u64;
        let hour = hh as u64;
        let minute = mm as u64;
        let second = ss as u64;
        let leap = is_leap_year(year);
        let (mut ydays, _mdays) = match month {
            1 => (0, 31),
            2 if leap => (31, 29),
            2 => (31, 28),
            3 => (59, 31),
            4 => (90, 30),
            5 => (120, 31),
            6 => (151, 30),
            7 => (181, 31),
            8 => (212, 31),
            9 => (243, 30),
            10 => (273, 31),
            11 => (304, 30),
            12 => (334, 31),
            _ => panic!(),
        };
        // if day > mdays || day == 0 {
        //     panic!();
        // }
        ydays += day - 1;
        if leap && month > 2 {
            ydays += 1;
        }

        let leap_years = ((year - 1) - 1968) / 4 - ((year - 1) - 1900) / 100 + ((year - 1) - 1600) / 400;
        let days = (year - 1970) * 365 + leap_years + ydays;

        let time = second + minute * 60 + hour * 3600;

        let total_seconds = time + days * 86400;
        UNIX_EPOCH + Duration::new(total_seconds, nsec)
    }

    pub fn systemtime_to_datetime(v: SystemTime) -> (i32, u8, u8, u8, u8, u8, u32) {
        let dur = v.duration_since(UNIX_EPOCH).expect("all times should be after the epoch");
        let secs_since_epoch = dur.as_secs();

        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        const DAYS_PER_400Y: i64 = 365 * 400 + 97;
        const DAYS_PER_100Y: i64 = 365 * 100 + 24;
        const DAYS_PER_4Y: i64 = 365 * 4 + 1;

        let days = (secs_since_epoch / 86400) as i64 - LEAPOCH;
        let secs_of_day = secs_since_epoch % 86400;

        let mut qc_cycles = days / DAYS_PER_400Y;
        let mut remdays = days % DAYS_PER_400Y;

        if remdays < 0 {
            remdays += DAYS_PER_400Y;
            qc_cycles -= 1;
        }

        let mut c_cycles = remdays / DAYS_PER_100Y;
        if c_cycles == 4 {
            c_cycles -= 1;
        }
        remdays -= c_cycles * DAYS_PER_100Y;

        let mut q_cycles = remdays / DAYS_PER_4Y;
        if q_cycles == 25 {
            q_cycles -= 1;
        }
        remdays -= q_cycles * DAYS_PER_4Y;

        let mut remyears = remdays / 365;
        if remyears == 4 {
            remyears -= 1;
        }
        remdays -= remyears * 365;

        let mut year = 2000 + remyears + 4 * q_cycles + 100 * c_cycles + 400 * qc_cycles;

        let months = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];
        let mut mon = 0;
        for mon_len in months.iter() {
            mon += 1;
            if remdays < *mon_len {
                break;
            }
            remdays -= *mon_len;
        }
        let mday = remdays + 1;
        let mon = if mon + 2 > 12 {
            year += 1;
            mon - 10
        } else {
            mon + 2
        };

        (
            year as i32,
            mon,
            mday as u8,
            (secs_of_day / 3600) as u8,
            (secs_of_day / 60 % 60) as u8,
            (secs_of_day % 60) as u8,
            dur.subsec_nanos(),
        )
    }
}

mod chrono {
    use chrono::{Datelike, Timelike};
    use std::time::SystemTime;

    pub fn rand_date() -> chrono::NaiveDate {
        let (y, m, d) = super::rand_date();
        chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32).unwrap()
    }

    #[inline]
    pub fn next_date(d: chrono::NaiveDate) -> chrono::NaiveDate {
        d.succ_opt().unwrap()
    }

    #[inline]
    pub fn prev_date(d: chrono::NaiveDate) -> chrono::NaiveDate {
        d.pred_opt().unwrap()
    }

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u8, u8) {
        let date = chrono::NaiveDate::from_num_days_from_ce_opt(n + 719162).unwrap();
        (date.year(), date.month() as u8, date.day() as u8)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u8, u8)) -> i32 {
        let days = chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32).unwrap().num_days_from_ce();
        days - 719162
    }

    #[inline]
    pub fn date_to_isoweekdate((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
        let date = chrono::NaiveDate::from_ymd_opt(y, m as u32, d as u32).unwrap();
        let iw = date.iso_week();
        let (y, w, d) = (iw.year(), iw.week(), date.weekday().number_from_monday());
        (y, w as u8, d as u8)
    }

    #[inline]
    pub fn isoweekdate_to_date((y, w, wd): (i32, u8, u8)) -> (i32, u8, u8) {
        let wd = chrono::Weekday::try_from(wd - 1).unwrap();
        let date = chrono::NaiveDate::from_isoywd_opt(y, w as u32, wd).unwrap();
        (date.year(), date.month() as u8, date.day() as u8)
    }

    #[inline]
    pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u8, u8, u8, u8, u8, u32)) -> SystemTime {
        chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
            .unwrap()
            .and_hms_nano_opt(hh as u32, mm as u32, ss as u32, nsec)
            .unwrap()
            .and_local_timezone(chrono::Utc)
            .unwrap()
            .into()
    }

    #[inline]
    pub fn systemtime_to_datetime(v: SystemTime) -> (i32, u8, u8, u8, u8, u8, u32) {
        let d: chrono::DateTime<chrono::Utc> = v.into();
        (
            d.year() as i32,
            d.month() as u8,
            d.day() as u8,
            d.hour() as u8,
            d.minute() as u8,
            d.second() as u8,
            d.nanosecond(),
        )
    }
}

mod time {
    use std::time::SystemTime;

    const UNIX_EPOCH_JULIAN_DAY: i32 = 2440588;

    pub fn rand_date() -> time::Date {
        let (y, m, d) = super::rand_date();
        time::Date::from_calendar_date(y, m.try_into().unwrap(), d).unwrap()
    }

    #[inline]
    pub fn next_date(d: time::Date) -> time::Date {
        d.next_day().unwrap()
    }

    #[inline]
    pub fn prev_date(d: time::Date) -> time::Date {
        d.previous_day().unwrap()
    }

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u8, u8) {
        let date = time::Date::from_julian_day(n + UNIX_EPOCH_JULIAN_DAY).unwrap();
        (date.year(), date.month() as u8, date.day() as u8)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u8, u8)) -> i32 {
        time::Date::from_calendar_date(y, time::Month::try_from(m as u8).unwrap(), d as u8)
            .unwrap()
            .to_julian_day()
            - UNIX_EPOCH_JULIAN_DAY
    }

    #[inline]
    pub fn date_to_isoweekdate((y, m, d): (i32, u8, u8)) -> (i32, u8, u8) {
        let date = time::Date::from_calendar_date(y, m.try_into().unwrap(), d).unwrap();
        let (y, w, wd) = date.to_iso_week_date();
        (y, w as u8, wd.number_from_monday())
    }

    #[inline]
    pub fn isoweekdate_to_date((y, w, wd): (i32, u8, u8)) -> (i32, u8, u8) {
        let d = time::Date::from_iso_week_date(y, w, time::Weekday::Sunday.nth_next(wd)).unwrap();
        (d.year(), d.month() as u8, d.day() as u8)
    }

    #[inline]
    pub fn datetime_to_systemtime((y, m, d, hh, mm, ss, nsec): (i32, u8, u8, u8, u8, u8, u32)) -> SystemTime {
        time::Date::from_calendar_date(y, time::Month::try_from(m as u8).unwrap(), d as u8)
            .unwrap()
            .with_hms_nano(hh, mm, ss, nsec)
            .unwrap()
            .assume_utc()
            .into()
    }

    #[inline]
    pub fn systemtime_to_datetime(v: SystemTime) -> (i32, u8, u8, u8, u8, u8, u32) {
        let d: time::OffsetDateTime = v.into();
        (
            d.year() as i32,
            d.month() as u8,
            d.day() as u8,
            d.hour() as u8,
            d.minute() as u8,
            d.second() as u8,
            d.nanosecond(),
        )
    }
}

mod hinnant {
    pub fn days_from_civil((y, m, d): (i32, u8, u8)) -> i32 {
        let y = y as i32 - (m <= 2) as i32;
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400) as u32;
        let doy = (153 * if m > 2 { (m - 3) as u32 } else { (m + 9) as u32 } + 2) / 5 + d as u32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe as i32 - 719468
    }

    pub fn days_from_civil_u((y, m, d): (i32, u8, u8)) -> i32 {
        let y = y as u32 - (m <= 2) as u32;
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400) as u32;
        let doy = (153 * if m > 2 { (m - 3) as u32 } else { (m + 9) as u32 } + 2) / 5 + d as u32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        (era * 146097 + doe as u32 - 719468) as i32
    }

    pub fn civil_from_days(n: i32) -> (i32, u8, u8) {
        let z = n + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as i32) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (y + (m <= 2) as i32, m as u8, d as u8)
    }

    pub fn civil_from_days_u(n: i32) -> (i32, u8, u8) {
        let z = (n + 719468) as u32;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as u32) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        ((y + (m <= 2) as u32) as i32, m as u8, d as u8)
    }
}

fn bench_rd_to_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_rd_to_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_date(black_box(rd))))
    });
    group.bench_function("hinnant_unsigned", |b| {
        b.iter_custom(bencher(rand_rd, |rd| hinnant::civil_from_days_u(black_box(rd))))
    });
    group.bench_function("hinnant", |b| {
        b.iter_custom(bencher(rand_rd, |rd| hinnant::civil_from_days(black_box(rd))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_rd, |rd| httpdate::rd_to_date(black_box(rd))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_rd, |rd| humantime::rd_to_date(black_box(rd))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_rd, |rd| chrono::rd_to_date(black_box(rd))))
    });
    group.bench_function("time", |b| b.iter_custom(bencher(rand_rd, |rd| time::rd_to_date(black_box(rd)))));
    group.finish();
}

fn bench_date_to_rd(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_rd");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_rd(black_box(d))))
    });
    group.bench_function("hinnant_unsigned", |b| {
        b.iter_custom(bencher(rand_date, |d| hinnant::days_from_civil_u(black_box(d))))
    });
    group.bench_function("hinnant", |b| {
        b.iter_custom(bencher(rand_date, |d| hinnant::days_from_civil(black_box(d))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_date, |d| httpdate::date_to_rd(black_box(d))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_date, |d| humantime::date_to_rd(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_date, |d| chrono::date_to_rd(black_box(d))))
    });
    group.bench_function("time", |b| b.iter_custom(bencher(rand_date, |d| time::date_to_rd(black_box(d)))));
    group.finish();
}

fn bench_rd_to_weekday(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_rd_to_weekday");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo::rd_to_weekday(black_box(rd))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo_alt::rd_to_weekday(black_box(rd))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_rd, |rd| datealgo_alt::rd_to_weekday2(black_box(rd))))
    });
    group.finish();
}

fn bench_date_to_weekday(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_weekday");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_weekday(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::date_to_weekday(black_box(d))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::date_to_weekday2(black_box(d))))
    });
    group.finish();
}

fn bench_next_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_next_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::next_date(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::next_date(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(chrono::rand_date, |d| chrono::next_date(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(time::rand_date, |d| time::next_date(black_box(d))))
    });
    group.finish();
}

fn bench_prev_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_prev_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::prev_date(black_box(d))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo_alt::prev_date(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(chrono::rand_date, |d| chrono::prev_date(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(time::rand_date, |d| time::prev_date(black_box(d))))
    });
    group.finish();
}

fn bench_secs_to_dhms(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_dhms");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_dhms(black_box(s))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo_alt::secs_to_dhms(black_box(s))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo_alt::secs_to_dhms2(black_box(s))))
    });
    group.finish();
}

fn bench_dhms_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_dhms_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dhms, |dhms| datealgo::dhms_to_secs(black_box(dhms))))
    });
    group.finish();
}

fn bench_secs_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_datetime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_secs, |s| datealgo::secs_to_datetime(black_box(s))))
    });
    group.finish();
}

fn bench_datetime_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_datetime_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dt, |dt| datealgo::datetime_to_secs(black_box(dt))))
    });
    group.finish();
}

fn bench_is_leap_year(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_is_leap_year");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo::is_leap_year(black_box(y))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year(black_box(y))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year2(black_box(y))))
    });
    group.bench_function("datealgo_alt3", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year3(black_box(y))))
    });
    group.bench_function("datealgo_alt4", |b| {
        b.iter_custom(bencher(rand_year, |y| datealgo_alt::is_leap_year4(black_box(y))))
    });
    group.finish();
}

fn bench_days_in_month(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_days_in_month");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo::days_in_month(black_box(y), black_box(m))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo_alt::days_in_month(black_box(y), black_box(m))))
    });
    group.bench_function("datealgo_alt2", |b| {
        b.iter_custom(bencher(rand_ym, |(y, m)| datealgo_alt::days_in_month2(black_box(y), black_box(m))))
    });
    group.finish();
}

fn bench_date_to_isoweekdate(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_date_to_isoweekdate");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_date, |d| datealgo::date_to_isoweekdate(black_box(d))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_date, |d| chrono::date_to_isoweekdate(black_box(d))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_date, |d| time::date_to_isoweekdate(black_box(d))))
    });
    group.finish();
}

fn bench_isoweekdate_to_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_isoweekdate_to_date");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| datealgo::isoweekdate_to_date(black_box(iwd))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| chrono::isoweekdate_to_date(black_box(iwd))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_iwd, |iwd| time::isoweekdate_to_date(black_box(iwd))))
    });
    group.finish();
}

fn bench_systemtime_to_secs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_systemtime_to_secs");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_secs(black_box(st))))
    });
    group.bench_function("datealgo_alt", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo_alt::systemtime_to_secs(black_box(st))))
    });
    group.finish();
}

fn bench_secs_to_systemtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_secs_to_systemtime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_sn, |sn| datealgo::secs_to_systemtime(black_box(sn))))
    });
    group.finish();
}

fn bench_systemtime_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_systemtime_to_datetime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_st, |st| datealgo::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_st, |st| httpdate::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_st, |st| humantime::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_st, |st| time::systemtime_to_datetime(black_box(st))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_st, |st| chrono::systemtime_to_datetime(black_box(st))))
    });
    group.finish();
}

fn bench_datetime_to_systemtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare_datetime_to_systemtime");
    group.bench_function("datealgo", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| datealgo::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("httpdate", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| httpdate::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("humantime", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| humantime::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("time", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| time::datetime_to_systemtime(black_box(dtn))))
    });
    group.bench_function("chrono", |b| {
        b.iter_custom(bencher(rand_dtn, |dtn| chrono::datetime_to_systemtime(black_box(dtn))))
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(5000).measurement_time(Duration::from_secs(10));
    targets =
        bench_rd_to_date,
        bench_date_to_rd,
        bench_rd_to_weekday,
        bench_date_to_weekday,
        bench_next_date,
        bench_prev_date,
        bench_secs_to_dhms,
        bench_dhms_to_secs,
        bench_secs_to_datetime,
        bench_datetime_to_secs,
        bench_is_leap_year,
        bench_days_in_month,
        bench_date_to_isoweekdate,
        bench_isoweekdate_to_date,
        bench_systemtime_to_secs,
        bench_secs_to_systemtime,
        bench_systemtime_to_datetime,
        bench_datetime_to_systemtime,
);
criterion_main!(benches);
