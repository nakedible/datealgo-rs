#![allow(clippy::absurd_extreme_comparisons, clippy::manual_range_contains)]

use std::time::SystemTime;

use quickcheck::{quickcheck, Arbitrary, Gen, TestResult};

#[derive(Debug, Clone, Copy)]
struct Val<const MIN: i128, const MAX: i128>(i128);

impl<const MIN: i128, const MAX: i128> Val<MIN, MAX> {
    fn i64(&self) -> i64 {
        assert!(self.0 >= i64::MIN as i128 && self.0 <= i64::MAX as i128);
        self.0 as i64
    }

    fn i32(&self) -> i32 {
        assert!(self.0 >= i32::MIN as i128 && self.0 <= i32::MAX as i128);
        self.0 as i32
    }

    fn u32(&self) -> u32 {
        assert!(self.0 >= u32::MIN as i128 && self.0 <= u32::MAX as i128);
        self.0 as u32
    }

    fn u8(&self) -> u8 {
        assert!(self.0 >= u8::MIN as i128 && self.0 <= u8::MAX as i128);
        self.0 as u8
    }
}

impl<const MIN: i128, const MAX: i128> Arbitrary for Val<MIN, MAX> {
    fn arbitrary(g: &mut Gen) -> Self {
        let v = i128::arbitrary(g).rem_euclid(MAX - MIN + 1) + MIN;
        Val(v)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let v = self.0;
        Box::new(v.shrink().map(Val))
    }
}

quickcheck! {
    fn quickcheck_rd_to_date(d: Val<-536895152, 536824295>) -> TestResult {
        let (y, m, d) = datealgo::rd_to_date(d.i32());
        assert!(y >= datealgo::YEAR_MIN && y <= datealgo::YEAR_MAX);
        assert!(m >= datealgo::consts::MONTH_MIN && m <= datealgo::consts::MONTH_MAX);
        assert!(d >= datealgo::consts::DAY_MIN && d <= datealgo::consts::DAY_MAX && d <= datealgo::days_in_month(y, m));
        TestResult::passed()
    }

    fn quickcheck_date_to_rd(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let rd = datealgo::date_to_rd((y.i32(), m.u8(), d.u8()));
        assert!(rd >= datealgo::RD_MIN && rd <= datealgo::RD_MAX);
        TestResult::passed()
    }

    fn quickcheck_rd_to_weekday(d: Val<-536895152, 536824295>) -> TestResult {
        let wd = datealgo::rd_to_weekday(d.i32());
        assert!(wd >= datealgo::consts::WEEKDAY_MIN && wd <= datealgo::consts::WEEKDAY_MAX);
        TestResult::passed()
    }

    fn quickcheck_date_to_weekday(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let wd = datealgo::date_to_weekday((y.i32(), m.u8(), d.u8()));
        assert!(wd >= datealgo::consts::WEEKDAY_MIN && wd <= datealgo::consts::WEEKDAY_MAX);
        TestResult::passed()
    }

    fn quickcheck_next_date(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        if y.i32() == datealgo::YEAR_MAX && m.u8() == datealgo::consts::MONTH_MAX && d.u8() == datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let (ny, nm, nd) = datealgo::next_date((y.i32(), m.u8(), d.u8()));
        assert!(ny >= datealgo::YEAR_MIN && ny <= datealgo::YEAR_MAX);
        assert!(nm >= datealgo::consts::MONTH_MIN && nm <= datealgo::consts::MONTH_MAX);
        assert!(nd >= datealgo::consts::DAY_MIN && nd <= datealgo::consts::DAY_MAX && nd <= datealgo::days_in_month(ny, nm));
        TestResult::passed()
    }

    fn quickcheck_prev_date(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        if y.i32() == datealgo::YEAR_MIN && m.u8() == datealgo::consts::MONTH_MIN && d.u8() == datealgo::consts::DAY_MIN {
            return TestResult::discard();
        }
        let (py, pm, pd) = datealgo::prev_date((y.i32(), m.u8(), d.u8()));
        assert!(py >= datealgo::YEAR_MIN && py <= datealgo::YEAR_MAX);
        assert!(pm >= datealgo::consts::MONTH_MIN && pm <= datealgo::consts::MONTH_MAX);
        assert!(pd >= datealgo::consts::DAY_MIN && pd <= datealgo::consts::DAY_MAX && pd <= datealgo::days_in_month(py, pm));
        TestResult::passed()
    }

    fn quickcheck_secs_to_dhms(s: Val<-46387741132800, 46381619174399 >) -> TestResult {
        let (d, h, m, s) = datealgo::secs_to_dhms(s.i64());
        assert!(d >= datealgo::RD_MIN && d <= datealgo::RD_MAX);
        assert!(h >= datealgo::consts::HOUR_MIN && h <= datealgo::consts::HOUR_MAX);
        assert!(m >= datealgo::consts::MINUTE_MIN && m <= datealgo::consts::MINUTE_MAX);
        assert!(s >= datealgo::consts::SECOND_MIN && s <= datealgo::consts::SECOND_MAX);
        TestResult::passed()
    }

    fn quickcheck_dhms_to_secs(d: Val<-536895152, 536824295>, h: Val<0, 23>, m: Val<0, 59>, s: Val<0, 59>) -> TestResult {
        let secs = datealgo::dhms_to_secs((d.i32(), h.u8(), m.u8(), s.u8()));
        assert!(secs >= datealgo::RD_SECONDS_MIN && secs <= datealgo::RD_SECONDS_MAX);
        TestResult::passed()
    }

    fn quickcheck_secs_to_datetime(s: Val<-46387741132800, 46381619174399 >) -> TestResult {
        let (y, m, d, h, min, sec) = datealgo::secs_to_datetime(s.i64());
        assert!(y >= datealgo::YEAR_MIN && y <= datealgo::YEAR_MAX);
        assert!(m >= datealgo::consts::MONTH_MIN && m <= datealgo::consts::MONTH_MAX);
        assert!(d >= datealgo::consts::DAY_MIN && d <= datealgo::consts::DAY_MAX && d <= datealgo::days_in_month(y, m));
        assert!(h >= datealgo::consts::HOUR_MIN && h <= datealgo::consts::HOUR_MAX);
        assert!(min >= datealgo::consts::MINUTE_MIN && min <= datealgo::consts::MINUTE_MAX);
        assert!(sec >= datealgo::consts::SECOND_MIN && sec <= datealgo::consts::SECOND_MAX);
        TestResult::passed()
    }

    fn quickcheck_datetime_to_secs(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>, h: Val<0, 23>, min: Val<0, 59>, sec: Val<0, 59>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let secs = datealgo::datetime_to_secs((y.i32(), m.u8(), d.u8(), h.u8(), min.u8(), sec.u8()));
        assert!(secs >= datealgo::RD_SECONDS_MIN && secs <= datealgo::RD_SECONDS_MAX);
        TestResult::passed()
    }

    fn quickcheck_is_leap_year(y: Val<-1467999, 1471744>) -> TestResult {
        let _ = datealgo::is_leap_year(y.i32());
        TestResult::passed()
    }

    fn quickcheck_days_in_month(y: Val<-1467999, 1471744>, m: Val<1, 12>) -> TestResult {
        let m = datealgo::days_in_month(y.i32(), m.u8());
        assert!(m >= 28 && m <= 31);
        TestResult::passed()
    }

    fn quickcheck_rd_to_isoweekdate(d: Val<-536895152, 536824295>) -> TestResult {
        let (y, w, wd) = datealgo::rd_to_isoweekdate(d.i32());
        assert!(y >= datealgo::YEAR_MIN && y <= datealgo::YEAR_MAX);
        assert!(w >= datealgo::consts::WEEK_MIN && w <= datealgo::consts::WEEK_MAX);
        assert!(wd >= datealgo::consts::WEEKDAY_MIN && wd <= datealgo::consts::WEEKDAY_MAX);
        TestResult::passed()
    }

    fn quickcheck_isoweekdate_to_rd(y: Val<-1467999, 1471744>, w: Val<1, 53>, wd: Val<1, 7>) -> TestResult {
        if w.u8() > datealgo::isoweeks_in_year(y.i32()) {
            return TestResult::discard();
        }
        let rd = datealgo::isoweekdate_to_rd((y.i32(), w.u8(), wd.u8()));
        assert!(rd >= datealgo::RD_MIN && rd <= datealgo::RD_MAX);
        TestResult::passed()
    }

    fn quickcheck_date_to_isoweekdate(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let (wy, ww, wd) = datealgo::date_to_isoweekdate((y.i32(), m.u8(), d.u8()));
        assert!(wy >= datealgo::YEAR_MIN && wy <= datealgo::YEAR_MAX);
        assert!(ww >= datealgo::consts::WEEK_MIN && ww <= datealgo::consts::WEEK_MAX);
        assert!(wd >= datealgo::consts::WEEKDAY_MIN && wd <= datealgo::consts::WEEKDAY_MAX);
        TestResult::passed()
    }

    fn quickcheck_isoweekdate_to_date(y: Val<-1467999, 1471744>, w: Val<1, 53>, wd: Val<1, 7>) -> TestResult {
        if w.u8() > datealgo::isoweeks_in_year(y.i32()) {
            return TestResult::discard();
        }
        if y.i32() == datealgo::YEAR_MAX && w.u8() == datealgo::isoweeks_in_year(y.i32()) && wd.u8() >= datealgo::consts::SATURDAY {
            return TestResult::discard();
        }
        let (dy, dm, dd) = datealgo::isoweekdate_to_date((y.i32(), w.u8(), wd.u8()));
        assert!(dy >= datealgo::YEAR_MIN && dy <= datealgo::YEAR_MAX);
        assert!(dm >= datealgo::consts::MONTH_MIN && dm <= datealgo::consts::MONTH_MAX);
        assert!(dd >= datealgo::consts::DAY_MIN && dd <= datealgo::consts::DAY_MAX && dd <= datealgo::days_in_month(dy, dm));
        TestResult::passed()
    }

    fn quickcheck_isoweeks_in_year(y: Val<-1467999, 1471744>) -> TestResult {
        let w = datealgo::isoweeks_in_year(y.i32());
        assert!(w >= 52 && w <= 53);
        TestResult::passed()
    }

    fn quickcheck_systemtime_to_secs(st: SystemTime) -> TestResult {
        let (secs, nsecs) = datealgo::systemtime_to_secs(st).unwrap();
        assert!(secs >= datealgo::RD_SECONDS_MIN && secs <= datealgo::RD_SECONDS_MAX);
        assert!(nsecs >= datealgo::consts::NANOSECOND_MIN && nsecs <= datealgo::consts::NANOSECOND_MAX);
        TestResult::passed()
    }

    fn quickcheck_secs_to_systemtime(secs: Val<-46387741132800, 46381619174399 >, nsecs: Val<0, 999_999_999>) -> TestResult {
        let st = datealgo::secs_to_systemtime((secs.i64(), nsecs.u32()));
        assert!(st.is_some());
        TestResult::passed()
    }

    fn quickcheck_systemtime_to_datetime(st: SystemTime) -> TestResult {
        let (y, m, d, h, min, sec, nsec) = datealgo::systemtime_to_datetime(st).unwrap();
        assert!(y >= datealgo::YEAR_MIN && y <= datealgo::YEAR_MAX);
        assert!(m >= datealgo::consts::MONTH_MIN && m <= datealgo::consts::MONTH_MAX);
        assert!(d >= datealgo::consts::DAY_MIN && d <= datealgo::consts::DAY_MAX && d <= datealgo::days_in_month(y, m));
        assert!(h >= datealgo::consts::HOUR_MIN && h <= datealgo::consts::HOUR_MAX);
        assert!(min >= datealgo::consts::MINUTE_MIN && min <= datealgo::consts::MINUTE_MAX);
        assert!(sec >= datealgo::consts::SECOND_MIN && sec <= datealgo::consts::SECOND_MAX);
        assert!(nsec >= datealgo::consts::NANOSECOND_MIN && nsec <= datealgo::consts::NANOSECOND_MAX);
        TestResult::passed()
    }

    fn quickcheck_datetime_to_systemtime(y: Val<-1467999, 1471744>, m: Val<1, 12>, d: Val<1, 31>, h: Val<0, 23>, min: Val<0, 59>, sec: Val<0, 59>, nsec: Val<0, 999_999_999>) -> TestResult {
        if d.u8() > datealgo::days_in_month(y.i32(), m.u8()) {
            return TestResult::discard();
        }
        let st = datealgo::datetime_to_systemtime((y.i32(), m.u8(), d.u8(), h.u8(), min.u8(), sec.u8(), nsec.u32()));
        assert!(st.is_some());
        TestResult::passed()
    }
}
