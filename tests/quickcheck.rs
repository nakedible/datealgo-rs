use datealgo::*;

use chrono::Datelike;
use quickcheck::{quickcheck, TestResult};

quickcheck! {
    fn quickcheck_rd_to_date(d: i32) -> TestResult {
        if d < RD_MIN || d > RD_MAX {
            return TestResult::discard();
        }
        let a = rd_to_date(d);
        let Some(date) = chrono::NaiveDate::from_num_days_from_ce_opt(d + 719163) else {
            return TestResult::discard();            
        };
        let b = (date.year(), date.month(), date.day());
        println!("{:?} {:?}", a, b);
        TestResult::from_bool(a == b)
    }
}
