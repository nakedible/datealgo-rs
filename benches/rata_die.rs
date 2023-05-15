use chrono::{Datelike, Timelike};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gmtime::*;

// fn test_chrono((y, m, d): (u16, u8, u8)) -> std::time::Duration {
//     const b = chrono::NaiveDate::from_ymd_opt(0, 3, 1).unwrap();
//     let d = chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap();

// }

fn httpdate_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
    let dur = v
        .duration_since(UNIX_EPOCH)
        .expect("all times should be after the epoch");
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

fn httpdate_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
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

fn chrono_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
    chrono::NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
        .unwrap()
        .and_hms_opt(hh as u32, mm as u32, ss as u32)
        .unwrap()
        .and_local_timezone(chrono::Utc)
        .unwrap()
        .into()
}

fn chrono_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
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

fn humantime_to_systemtime((y, m, d, hh, mm, ss): (i16, u8, u8, u8, u8, u8)) -> SystemTime {
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

fn humantime_from_systemtime(v: SystemTime) -> (i16, u8, u8, u8, u8, u8, u8) {
    let dur = v
        .duration_since(UNIX_EPOCH)
        .expect("all times should be after the epoch");
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

fn days_from_civil((y, m, d): (i16, u8, u8)) -> i32 {
    let y = y as i32 - (m <= 2) as i32;
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u32;
    let doy = (153 * if m > 2 { (m - 3) as u32 } else { (m + 9) as u32 } + 2) / 5 + d as u32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i32 - 719468
}

fn civil_from_days(n: u32) -> (i16, u8, u8) {
    let z = n + 719468;
    let era = z.div_euclid(146097);
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe  + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y as i16 + (m <= 2) as i16, m as u8, d as u8) 
}

fn build_secs() -> [u64; 1000] {
    let mut rng = ChaChaRng::seed_from_u64(1970);
    let arr = [(); 1000].map(|_| rng.gen_range(0..(36525 * 86400)));
    arr
}

fn build_systemtimes() -> [SystemTime; 1000] {
    let mut rng = ChaChaRng::seed_from_u64(1970);
    let arr = [(); 1000].map(|_| UNIX_EPOCH + Duration::from_secs(rng.gen_range(0..(36525 * 86400))));
    arr
}

fn build_datetimes() -> [(i16, u8, u8, u8, u8, u8); 1000] {
    let mut rng = ChaChaRng::seed_from_u64(1970);
    let arr = [(); 1000].map(|_| {
        (
            rng.gen_range(1970..=2069),
            rng.gen_range(1..=12),
            rng.gen_range(1..=28),
            rng.gen_range(0..=23),
            rng.gen_range(0..=59),
            rng.gen_range(0..=59),
        )
    });
    arr
}

fn build_rata_die() -> [u32; 1000] {
    let mut rng = ChaChaRng::seed_from_u64(1970);
    let arr = [(); 1000].map(|_| UNIX_EPOCH_RATA_DIE + rng.gen_range(0..36525));
    arr
}

fn build_gregorian_dates() -> [(i16, u8, u8); 1000] {
    let mut rng = ChaChaRng::seed_from_u64(1970);
    let arr = [(); 1000].map(|_| (rng.gen_range(1970..=2069), rng.gen_range(1..=12), rng.gen_range(1..=28)));
    arr
}

fn bench_datetime_to_systemtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("datetime_to_systemtime");
    let arr = build_datetimes();
    group.bench_with_input(BenchmarkId::new("to_systemtime", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(to_systemtime))
    });
    group.bench_with_input(BenchmarkId::new("httpdate_to_systemtime", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(httpdate_to_systemtime))
    });
    group.bench_with_input(
        BenchmarkId::new("humantime_to_systemtime", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(humantime_to_systemtime)),
    );
    group.bench_with_input(BenchmarkId::new("chrono_to_systemtime", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(chrono_to_systemtime))
    });
    group.finish();
}

fn bench_systemtime_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("systemtime_to_datetime");
    let arr = build_systemtimes();
    group.bench_with_input(BenchmarkId::new("from_systemtime", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(from_systemtime))
    });
    group.bench_with_input(
        BenchmarkId::new("httpdate_from_systemtime", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(httpdate_from_systemtime)),
    );
    group.bench_with_input(
        BenchmarkId::new("humantime_from_systemtime", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(humantime_from_systemtime)),
    );
    group.bench_with_input(BenchmarkId::new("chrono_from_systemtime", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(chrono_from_systemtime))
    });
    group.finish();
}

fn bench_secs_to_dhms(c: &mut Criterion) {
    let mut group = c.benchmark_group("secs_to_dhms");
    let arr = build_secs();
    group.bench_with_input(BenchmarkId::new("secs_to_dhms", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(secs_to_dhms))
    });
    group.bench_with_input(BenchmarkId::new("secs_to_dhms2", "epoch+100"), &arr, |b, i| {
        b.iter(|| i.map(secs_to_dhms2))
    });
    group.finish();
}

fn bench_rata_die_to_gregorian_date(c: &mut Criterion) {
    let mut group = c.benchmark_group("rata_die_to_gregorian_date");
    let arr = build_rata_die();
    group.bench_with_input(
        BenchmarkId::new("rata_die_to_gregorian_date", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(rata_die_to_gregorian_date)),
    );
    group.bench_with_input(
        BenchmarkId::new("civil_from_days", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(civil_from_days)),
    );
    group.finish();
}

fn bench_gregorian_date_to_rata_die(c: &mut Criterion) {
    let mut group = c.benchmark_group("gregorian_date_to_rata_die");
    let arr = build_gregorian_dates();
    group.bench_with_input(
        BenchmarkId::new("gregorian_date_to_rata_die", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(gregorian_date_to_rata_die)),
    );
    group.bench_with_input(
        BenchmarkId::new("days_from_civil", "epoch+100"),
        &arr,
        |b, i| b.iter(|| i.map(days_from_civil)),
    );
    group.finish();
}

criterion_group!(
    benches,
    bench_datetime_to_systemtime,
    bench_systemtime_to_datetime,
    bench_secs_to_dhms,
    bench_rata_die_to_gregorian_date,
    bench_gregorian_date_to_rata_die,
);
criterion_main!(benches);
