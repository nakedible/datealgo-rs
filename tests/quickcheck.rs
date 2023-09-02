use datealgo::*;

use quickcheck::{quickcheck, TestResult};

quickcheck! {
    fn quickcheck_rd_to_date(d: time::Date) -> TestResult {
        let rd = d.to_julian_day() - 2440588;
        let a = rd_to_date(rd);
        let b = (d.year() as i32, d.month() as u8, d.day() as u8);
        TestResult::from_bool(a == b)
    }

    fn quickcheck_date_to_rd(d: time::Date) -> TestResult {
        let a = (d.year() as i32, d.month() as u8, d.day() as u8);
        let rd = date_to_rd(a);
        TestResult::from_bool(rd == d.to_julian_day() - 2440588)
    }

    fn quickcheck_rd_to_weekday(d: time::Date) -> TestResult {
        let rd = d.to_julian_day() - 2440588;
        let wd_a = rd_to_weekday(rd);
        let wd_b = d.weekday().number_from_monday();
        TestResult::from_bool(wd_a as u8 == wd_b)
    }

    fn quickcheck_date_to_weekday(d: time::Date) -> TestResult {
        let rd = d.to_julian_day() - 2440588;
        let a = rd_to_date(rd);
        let wd_a = date_to_weekday(a);
        let wd_b = d.weekday().number_from_monday();
        TestResult::from_bool(wd_a as u8 == wd_b)
    }

    fn quickcheck_is_leap_year(y: i32) -> TestResult {
        if y < datealgo::YEAR_MIN || y > datealgo::YEAR_MAX {
            return TestResult::discard();
        }
        let leap_a = datealgo::is_leap_year(y);
        let leap_b = time::util::is_leap_year(y);
        TestResult::from_bool(leap_a == leap_b)
    }

    fn quickcheck_days_in_month(y: i32, m: u8) -> TestResult {
        if y < datealgo::YEAR_MIN || y > datealgo::YEAR_MAX {
            return TestResult::discard();
        }
        if m < datealgo::consts::MONTH_MIN || m > datealgo::consts::MONTH_MAX {
            return TestResult::discard();
        }
        let days_a = datealgo::days_in_month(y, m);
        let days_b = time::util::days_in_year_month(y, m.try_into().unwrap());
        TestResult::from_bool(days_a == days_b)
    }

    fn quickcheck_systemtime_to_datetime(s: time::PrimitiveDateTime) -> TestResult {
        let s = s.assume_utc();
        let a = systemtime_to_datetime(s.into()).unwrap();
        let b = (
            s.year() as i32,
            s.month() as u8,
            s.day() as u8,
            s.hour() as u8,
            s.minute() as u8,
            s.second() as u8,
            s.nanosecond(),
        );
        TestResult::from_bool(a == b)
    }

    fn quickcheck_datetime_to_systemtime(s: time::PrimitiveDateTime) -> TestResult {
        let s = s.assume_utc();
        let dt = (
            s.year() as i32,
            s.month() as u8,
            s.day() as u8,
            s.hour() as u8,
            s.minute() as u8,
            s.second() as u8,
            s.nanosecond(),
        );
        let a = datetime_to_systemtime(dt).unwrap();
        let b: std::time::SystemTime = s.into();
        TestResult::from_bool(a == b)
    }
}
