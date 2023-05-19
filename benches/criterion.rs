use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

mod datealgo_alt {
    #[inline]
    pub const fn secs_to_dhms2(secs: i64) -> (i32, u8, u8, u8) {
        let secs = secs as u64;
        let ss = secs % 60;
        let secs = secs / 60;
        let mm = secs % 60;
        let secs = secs / 60;
        let hh = secs % 24;
        let secs = secs / 24;
        (secs as i32, hh as u8, mm as u8, ss as u8)
    }
}

mod httpdate {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u32, u32) {
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

        (year as i32, mon as u32, mday as u32)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u32, u32)) -> i32 {
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

    pub fn httpdate_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
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

        let mut wday = (3 + days) % 7;
        if wday <= 0 {
            wday += 7
        };
        (
            year as i16,
            mon as u8,
            mday as u8,
            (secs_of_day / 3600) as u8,
            ((secs_of_day % 3600) / 60) as u8,
            (secs_of_day % 60) as u8,
            wday as u8,
        )
    }

    pub fn httpdate_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
        fn is_leap_year(y: i16) -> bool {
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
        UNIX_EPOCH + Duration::from_secs(ss as u64 + mm as u64 * 60 + hh as u64 * 3600 + days * 86400)
    }
}

mod humantime {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u32, u32) {
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

        (year as i32, mon as u32, mday as u32)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u32, u32)) -> i32 {
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

    pub fn humantime_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
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
        UNIX_EPOCH + Duration::from_secs(total_seconds)
    }

    pub fn humantime_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
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
            year as i16,
            mon,
            mday as u8,
            (secs_of_day / 3600) as u8,
            (secs_of_day / 60 % 60) as u8,
            (secs_of_day % 60) as u8,
            1,
        )
    }
}

mod chrono {
    use chrono::{Datelike, Timelike};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u32, u32) {
        let date = chrono::NaiveDate::from_num_days_from_ce_opt(n + 719162).unwrap();
        (date.year(), date.month(), date.day())
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u32, u32)) -> i32 {
        let days = chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap().num_days_from_ce();
        days - 719162
    }

    pub fn chrono_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
        chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
            .unwrap()
            .and_hms_opt(hh as u32, mm as u32, ss as u32)
            .unwrap()
            .and_local_timezone(chrono::Utc)
            .unwrap()
            .into()
    }

    pub fn chrono_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
        let d: chrono::DateTime<chrono::Utc> = v.into();
        (
            d.year() as i16,
            d.month() as u8,
            d.day() as u8,
            d.hour() as u8,
            d.minute() as u8,
            d.second() as u8,
            d.weekday().number_from_monday() as u8,
        )
    }
}

mod time {
    const UNIX_EPOCH_JULIAN_DAY: i32 = 2440588;

    #[inline]
    pub fn rd_to_date(n: i32) -> (i32, u32, u32) {
        let date = time::Date::from_julian_day(n + UNIX_EPOCH_JULIAN_DAY).unwrap();
        (date.year(), date.month() as u32, date.day() as u32)
    }

    #[inline]
    pub fn date_to_rd((y, m, d): (i32, u32, u32)) -> i32 {
        time::Date::from_calendar_date(y, time::Month::try_from(m as u8).unwrap(), d as u8)
            .unwrap()
            .to_julian_day()
            - UNIX_EPOCH_JULIAN_DAY
    }
}

mod hinnant {
    pub fn days_from_civil((y, m, d): (i32, u32, u32)) -> i32 {
        let y = y as i32 - (m <= 2) as i32;
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400) as u32;
        let doy = (153 * if m > 2 { (m - 3) as u32 } else { (m + 9) as u32 } + 2) / 5 + d as u32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe as i32 - 719468
    }

    pub fn days_from_civil_u((y, m, d): (i32, u32, u32)) -> i32 {
        let y = y as u32 - (m <= 2) as u32;
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400) as u32;
        let doy = (153 * if m > 2 { (m - 3) as u32 } else { (m + 9) as u32 } + 2) / 5 + d as u32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        (era * 146097 + doe as u32 - 719468) as i32
    }

    pub fn civil_from_days(n: i32) -> (i32, u32, u32) {
        let z = n + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as i32) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (y + (m <= 2) as i32, m as u32, d as u32)
    }

    pub fn civil_from_days_u(n: i32) -> (i32, u32, u32) {
        let z = (n + 719468) as u32;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as u32) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        ((y + (m <= 2) as u32) as i32, m as u32, d as u32)
    }
}

// fn bench_datetime_to_systemtime(c: &mut Criterion) {
//     let mut group = c.benchmark_group("datetime_to_systemtime");
//     let arr = build_datetimes();
//     group.bench_with_input(BenchmarkId::new("to_systemtime", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(to_systemtime))
//     });
//     group.bench_with_input(BenchmarkId::new("httpdate_to_systemtime", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(httpdate_to_systemtime))
//     });
//     group.bench_with_input(
//         BenchmarkId::new("humantime_to_systemtime", "epoch+100"),
//         &arr,
//         |b, i| b.iter(|| i.map(humantime_to_systemtime)),
//     );
//     group.bench_with_input(BenchmarkId::new("chrono_to_systemtime", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(chrono_to_systemtime))
//     });
//     group.finish();
// }

// fn bench_systemtime_to_datetime(c: &mut Criterion) {
//     let mut group = c.benchmark_group("systemtime_to_datetime");
//     let arr = build_systemtimes();
//     group.bench_with_input(BenchmarkId::new("from_systemtime", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(from_systemtime))
//     });
//     group.bench_with_input(
//         BenchmarkId::new("httpdate_from_systemtime", "epoch+100"),
//         &arr,
//         |b, i| b.iter(|| i.map(httpdate_from_systemtime)),
//     );
//     group.bench_with_input(
//         BenchmarkId::new("humantime_from_systemtime", "epoch+100"),
//         &arr,
//         |b, i| b.iter(|| i.map(humantime_from_systemtime)),
//     );
//     group.bench_with_input(BenchmarkId::new("chrono_from_systemtime", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(chrono_from_systemtime))
//     });
//     group.finish();
// }

// fn bench_secs_to_dhms(c: &mut Criterion) {
//     let mut group = c.benchmark_group("secs_to_dhms");
//     let arr = build_secs();
//     group.bench_with_input(BenchmarkId::new("secs_to_dhms", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(secs_to_dhms))
//     });
//     group.bench_with_input(BenchmarkId::new("secs_to_dhms2", "epoch+100"), &arr, |b, i| {
//         b.iter(|| i.map(secs_to_dhms2))
//     });
//     group.finish();
// }

fn bench_rd_to_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("rd_to_date");
    let rd = datealgo::date_to_rd((2023, 5, 12));
    group.bench_with_input(BenchmarkId::new("datealgo", rd), &rd, |b, i| {
        b.iter(|| black_box(datealgo::rd_to_date(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("hinnant_unsigned", rd), &rd, |b, i| {
        b.iter(|| black_box(hinnant::civil_from_days_u(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("hinnant", rd), &rd, |b, i| {
        b.iter(|| black_box(hinnant::civil_from_days(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("httpdate", rd), &rd, |b, i| {
        b.iter(|| black_box(httpdate::rd_to_date(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("humantime", rd), &rd, |b, i| {
        b.iter(|| black_box(humantime::rd_to_date(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("chrono", rd), &rd, |b, i| {
        b.iter(|| black_box(chrono::rd_to_date(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("time", rd), &rd, |b, i| {
        b.iter(|| black_box(time::rd_to_date(black_box(*i))))
    });
    group.finish();
}

fn bench_date_to_rd(c: &mut Criterion) {
    let mut group = c.benchmark_group("date_to_rd");
    let d = (2023, 5, 12);
    group.bench_with_input(BenchmarkId::new("datealgo", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(datealgo::date_to_rd(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("hinnant_unsigned", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(hinnant::days_from_civil_u(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("hinnant", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(hinnant::days_from_civil(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("httpdate", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(httpdate::date_to_rd(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("humantime", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(humantime::date_to_rd(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("chrono", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(chrono::date_to_rd(black_box(*i))))
    });
    group.bench_with_input(BenchmarkId::new("time", format!("{:?}", d)), &d, |b, i| {
        b.iter(|| black_box(time::date_to_rd(black_box(*i))))
    });
    group.finish();
}

criterion_group!(
    benches,
    // bench_datetime_to_systemtime,
    // bench_systemtime_to_datetime,
    // bench_secs_to_dhms,
    bench_rd_to_date,
    bench_date_to_rd,
);
criterion_main!(benches);
