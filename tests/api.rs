#[rustfmt::skip]
    
use datealgo::*;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn test_consts() {
    assert_eq!(RD_MIN, -536895152);
    assert_eq!(RD_MAX, 536824295);
    assert_eq!(RD_SECONDS_MIN, -46387741132800);
    assert_eq!(RD_SECONDS_MAX, 46381619174399);
}

#[test]
fn test_date_to_rd() {
    assert_eq!(date_to_rd((0, 3, 1)), -719468);
    assert_eq!(date_to_rd((1970, 1, 1)), 0);
    assert_eq!(date_to_rd((i16::MIN as i32, 1, 1)), -12687794);
    assert_eq!(date_to_rd((i16::MAX as i32, 12, 31)), 11248737);
    assert_eq!(date_to_rd((i16::MIN as i32 - 1, 1, 1)), -12688159);
    assert_eq!(date_to_rd((i16::MAX as i32 + 1, 12, 31)), 11249103);
}

#[test]
fn test_rd_to_date() {
    assert_eq!(rd_to_date(-719468), (0, 3, 1));
    assert_eq!(rd_to_date(0), (1970, 1, 1));
    assert_eq!(rd_to_date(-12687794), (i16::MIN as i32, 1, 1));
    assert_eq!(rd_to_date(11248737), (i16::MAX as i32, 12, 31));
    assert_eq!(rd_to_date(-12687795), (i16::MIN as i32 - 1, 12, 31));
    assert_eq!(rd_to_date(11248738), (i16::MAX as i32 + 1, 1, 1));
}

#[test]
fn test_rd_to_weekday() {
    assert_eq!(rd_to_weekday(RD_MIN), 1);
    assert_eq!(rd_to_weekday(RD_MAX), 4);
    assert_eq!(rd_to_weekday(-719468), 3);
    assert_eq!(rd_to_weekday(-4), 7);
    assert_eq!(rd_to_weekday(-3), 1);
    assert_eq!(rd_to_weekday(-2), 2);
    assert_eq!(rd_to_weekday(-1), 3);
    assert_eq!(rd_to_weekday(0), 4);
    assert_eq!(rd_to_weekday(1), 5);
    assert_eq!(rd_to_weekday(2), 6);
    assert_eq!(rd_to_weekday(3), 7);
    assert_eq!(rd_to_weekday(4), 1);
    assert_eq!(rd_to_weekday(5), 2);
    assert_eq!(rd_to_weekday(6), 3);
    assert_eq!(rd_to_weekday(19489), 5);
}

#[test]
fn test_date_to_weekday() {
    assert_eq!(date_to_weekday((1970, 1, 1)), 4);
    assert_eq!(date_to_weekday((2023, 1, 1)), 7);
    assert_eq!(date_to_weekday((2023, 2, 1)), 3);
    assert_eq!(date_to_weekday((2023, 3, 1)), 3);
    assert_eq!(date_to_weekday((2023, 4, 1)), 6);
    assert_eq!(date_to_weekday((2023, 5, 1)), 1);
    assert_eq!(date_to_weekday((2023, 6, 1)), 4);
    assert_eq!(date_to_weekday((2023, 7, 1)), 6);
    assert_eq!(date_to_weekday((2023, 8, 1)), 2);
    assert_eq!(date_to_weekday((2023, 9, 1)), 5);
    assert_eq!(date_to_weekday((2023, 10, 1)), 7);
    assert_eq!(date_to_weekday((2023, 11, 1)), 3);
    assert_eq!(date_to_weekday((2023, 12, 1)), 5);
    assert_eq!(date_to_weekday((2023, 2, 28)), 2);
    assert_eq!(date_to_weekday((2020, 2, 29)), 6);
    assert_eq!(date_to_weekday((0, 1, 1)), 6);
    assert_eq!(date_to_weekday((-1, 1, 1)), 5);
    assert_eq!(date_to_weekday((-4, 1, 1)), 1);
    assert_eq!(date_to_weekday((-100, 1, 1)), 1);
    assert_eq!(date_to_weekday((-400, 1, 1)), 6);
}

#[test]
fn test_secs_to_dhms() {
    assert_eq!(secs_to_dhms(RD_SECONDS_MIN), (RD_MIN, 0, 0, 0));
    assert_eq!(secs_to_dhms(RD_SECONDS_MAX), (RD_MAX, 23, 59, 59));
}

#[test]
fn test_dhms_to_secs() {
    assert_eq!(dhms_to_secs((RD_MIN, 0, 0, 0)), RD_SECONDS_MIN);
    assert_eq!(dhms_to_secs((RD_MAX, 23, 59, 59)), RD_SECONDS_MAX);
}

#[test]
fn test_secs_to_datetime() {
    assert_eq!(secs_to_datetime(RD_SECONDS_MIN), (YEAR_MIN, 1, 1, 0, 0, 0));
    assert_eq!(secs_to_datetime(RD_SECONDS_MAX), (YEAR_MAX, 12, 31, 23, 59, 59));
}

#[test]
fn test_datetime_to_secs() {
    assert_eq!(datetime_to_secs((YEAR_MIN, 1, 1, 0, 0, 0)), RD_SECONDS_MIN);
    assert_eq!(datetime_to_secs((YEAR_MAX, 12, 31, 23, 59, 59)), RD_SECONDS_MAX);
}

#[test]
fn test_is_leap_year() {
    assert_eq!(is_leap_year(0), true);
    assert_eq!(is_leap_year(1), false);
    assert_eq!(is_leap_year(4), true);
    assert_eq!(is_leap_year(100), false);
    assert_eq!(is_leap_year(400), true);
    assert_eq!(is_leap_year(-1), false);
    assert_eq!(is_leap_year(-4), true);
    assert_eq!(is_leap_year(-100), false);
    assert_eq!(is_leap_year(-400), true);
}

#[test]
fn test_days_in_month() {
    assert_eq!(days_in_month(1, 1), 31);
    assert_eq!(days_in_month(1, 2), 28);
    assert_eq!(days_in_month(1, 3), 31);
    assert_eq!(days_in_month(1, 4), 30);
    assert_eq!(days_in_month(1, 5), 31);
    assert_eq!(days_in_month(1, 6), 30);
    assert_eq!(days_in_month(1, 7), 31);
    assert_eq!(days_in_month(1, 8), 31);
    assert_eq!(days_in_month(1, 9), 30);
    assert_eq!(days_in_month(1, 10), 31);
    assert_eq!(days_in_month(1, 11), 30);
    assert_eq!(days_in_month(1, 12), 31);
    assert_eq!(days_in_month(0, 1), 31);
    assert_eq!(days_in_month(0, 2), 29);
    assert_eq!(days_in_month(0, 3), 31);
    assert_eq!(days_in_month(0, 4), 30);
    assert_eq!(days_in_month(0, 5), 31);
    assert_eq!(days_in_month(0, 6), 30);
    assert_eq!(days_in_month(0, 7), 31);
    assert_eq!(days_in_month(0, 8), 31);
    assert_eq!(days_in_month(0, 9), 30);
    assert_eq!(days_in_month(0, 10), 31);
    assert_eq!(days_in_month(0, 11), 30);
    assert_eq!(days_in_month(0, 12), 31);
    assert_eq!(days_in_month(-1, 1), 31);
    assert_eq!(days_in_month(-1, 2), 28);
    assert_eq!(days_in_month(-1, 3), 31);
    assert_eq!(days_in_month(-1, 4), 30);
    assert_eq!(days_in_month(-1, 5), 31);
    assert_eq!(days_in_month(-1, 6), 30);
    assert_eq!(days_in_month(-1, 7), 31);
    assert_eq!(days_in_month(-1, 8), 31);
    assert_eq!(days_in_month(-1, 9), 30);
    assert_eq!(days_in_month(-1, 10), 31);
    assert_eq!(days_in_month(-1, 11), 30);
    assert_eq!(days_in_month(-1, 12), 31);
    assert_eq!(days_in_month(-4, 1), 31);
    assert_eq!(days_in_month(-4, 2), 29);
    assert_eq!(days_in_month(-4, 3), 31);
    assert_eq!(days_in_month(-4, 4), 30);
    assert_eq!(days_in_month(-4, 5), 31);
    assert_eq!(days_in_month(-4, 6), 30);
    assert_eq!(days_in_month(-4, 7), 31);
    assert_eq!(days_in_month(-4, 8), 31);
    assert_eq!(days_in_month(-4, 9), 30);
    assert_eq!(days_in_month(-4, 10), 31);
    assert_eq!(days_in_month(-4, 11), 30);
    assert_eq!(days_in_month(-4, 12), 31);
}

#[test]
fn test_rd_to_isoweekdate() {
    assert_eq!(rd_to_isoweekdate(date_to_rd((-4, 12, 30))), (-3, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((-4, 12, 31))), (-3, 1, 2));
    assert_eq!(rd_to_isoweekdate(date_to_rd((-3, 1, 1))), (-3, 1, 3));
    assert_eq!(rd_to_isoweekdate(date_to_rd((-1, 12, 31))), (-1, 52, 5));
    assert_eq!(rd_to_isoweekdate(date_to_rd((0, 1, 1))), (-1, 52, 6));
    assert_eq!(rd_to_isoweekdate(date_to_rd((0, 1, 2))), (-1, 52, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((0, 1, 3))), (0, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1970, 1, 1))), (1970, 1, 4));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1977, 1, 1))), (1976, 53, 6));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1977, 1, 2))), (1976, 53, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1977, 12, 31))), (1977, 52, 6));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1978, 1, 1))), (1977, 52, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1978, 1, 2))), (1978, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1978, 12, 31))), (1978, 52, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1979, 1, 1))), (1979, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1979, 12, 30))), (1979, 52, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1979, 12, 31))), (1980, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1980, 1, 1))), (1980, 1, 2));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1980, 12, 28))), (1980, 52, 7));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1980, 12, 29))), (1981, 1, 1));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1980, 12, 30))), (1981, 1, 2));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1980, 12, 31))), (1981, 1, 3));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1981, 1, 1))), (1981, 1, 4));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1981, 12, 31))), (1981, 53, 4));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1982, 1, 1))), (1981, 53, 5));
    assert_eq!(rd_to_isoweekdate(date_to_rd((1982, 1, 2))), (1981, 53, 6));
}

#[test]
fn test_isoweekdate_to_rd() {
    assert_eq!(isoweekdate_to_rd((-3, 1, 1)), date_to_rd((-4, 12, 30)));
    assert_eq!(isoweekdate_to_rd((-3, 1, 2)), date_to_rd((-4, 12, 31)));
    assert_eq!(isoweekdate_to_rd((-3, 1, 3)), date_to_rd((-3, 1, 1)));
    assert_eq!(isoweekdate_to_rd((-1, 52, 5)), date_to_rd((-1, 12, 31)));
    assert_eq!(isoweekdate_to_rd((-1, 52, 6)), date_to_rd((0, 1, 1)));
    assert_eq!(isoweekdate_to_rd((-1, 52, 7)), date_to_rd((0, 1, 2)));
    assert_eq!(isoweekdate_to_rd((0, 1, 1)), date_to_rd((0, 1, 3)));
    assert_eq!(isoweekdate_to_rd((1970, 1, 4)), date_to_rd((1970, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1976, 53, 6)), date_to_rd((1977, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1976, 53, 7)), date_to_rd((1977, 1, 2)));
    assert_eq!(isoweekdate_to_rd((1977, 52, 6)), date_to_rd((1977, 12, 31)));
    assert_eq!(isoweekdate_to_rd((1977, 52, 7)), date_to_rd((1978, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1978, 1, 1)), date_to_rd((1978, 1, 2)));
    assert_eq!(isoweekdate_to_rd((1978, 52, 7)), date_to_rd((1978, 12, 31)));
    assert_eq!(isoweekdate_to_rd((1979, 1, 1)), date_to_rd((1979, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1979, 52, 7)), date_to_rd((1979, 12, 30)));
    assert_eq!(isoweekdate_to_rd((1980, 1, 1)), date_to_rd((1979, 12, 31)));
    assert_eq!(isoweekdate_to_rd((1980, 1, 2)), date_to_rd((1980, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1980, 52, 7)), date_to_rd((1980, 12, 28)));
    assert_eq!(isoweekdate_to_rd((1981, 1, 1)), date_to_rd((1980, 12, 29)));
    assert_eq!(isoweekdate_to_rd((1981, 1, 2)), date_to_rd((1980, 12, 30)));
    assert_eq!(isoweekdate_to_rd((1981, 1, 3)), date_to_rd((1980, 12, 31)));
    assert_eq!(isoweekdate_to_rd((1981, 1, 4)), date_to_rd((1981, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1981, 53, 4)), date_to_rd((1981, 12, 31)));
    assert_eq!(isoweekdate_to_rd((1981, 53, 5)), date_to_rd((1982, 1, 1)));
    assert_eq!(isoweekdate_to_rd((1981, 53, 6)), date_to_rd((1982, 1, 2)));
}

#[test]
fn test_date_to_isoweekdate() {
    assert_eq!(date_to_isoweekdate((-4, 12, 30)), (-3, 1, 1));
    assert_eq!(date_to_isoweekdate((-4, 12, 31)), (-3, 1, 2));
    assert_eq!(date_to_isoweekdate((-3, 1, 1)), (-3, 1, 3));
    assert_eq!(date_to_isoweekdate((-1, 12, 31)), (-1, 52, 5));
    assert_eq!(date_to_isoweekdate((0, 1, 1)), (-1, 52, 6));
    assert_eq!(date_to_isoweekdate((0, 1, 2)), (-1, 52, 7));
    assert_eq!(date_to_isoweekdate((0, 1, 3)), (0, 1, 1));
    assert_eq!(date_to_isoweekdate((1970, 1, 1)), (1970, 1, 4));
    assert_eq!(date_to_isoweekdate((1977, 1, 1)), (1976, 53, 6));
    assert_eq!(date_to_isoweekdate((1977, 1, 2)), (1976, 53, 7));
    assert_eq!(date_to_isoweekdate((1977, 12, 31)), (1977, 52, 6));
    assert_eq!(date_to_isoweekdate((1978, 1, 1)), (1977, 52, 7));
    assert_eq!(date_to_isoweekdate((1978, 1, 2)), (1978, 1, 1));
    assert_eq!(date_to_isoweekdate((1978, 12, 31)), (1978, 52, 7));
    assert_eq!(date_to_isoweekdate((1979, 1, 1)), (1979, 1, 1));
    assert_eq!(date_to_isoweekdate((1979, 12, 30)), (1979, 52, 7));
    assert_eq!(date_to_isoweekdate((1979, 12, 31)), (1980, 1, 1));
    assert_eq!(date_to_isoweekdate((1980, 1, 1)), (1980, 1, 2));
    assert_eq!(date_to_isoweekdate((1980, 12, 28)), (1980, 52, 7));
    assert_eq!(date_to_isoweekdate((1980, 12, 29)), (1981, 1, 1));
    assert_eq!(date_to_isoweekdate((1980, 12, 30)), (1981, 1, 2));
    assert_eq!(date_to_isoweekdate((1980, 12, 31)), (1981, 1, 3));
    assert_eq!(date_to_isoweekdate((1981, 1, 1)), (1981, 1, 4));
    assert_eq!(date_to_isoweekdate((1981, 12, 31)), (1981, 53, 4));
    assert_eq!(date_to_isoweekdate((1982, 1, 1)), (1981, 53, 5));
    assert_eq!(date_to_isoweekdate((1982, 1, 2)), (1981, 53, 6));
}

#[test]
fn test_isoweekdate_to_date() {
    assert_eq!(isoweekdate_to_date((-3, 1, 1)), (-4, 12, 30));
    assert_eq!(isoweekdate_to_date((-3, 1, 2)), (-4, 12, 31));
    assert_eq!(isoweekdate_to_date((-3, 1, 3)), (-3, 1, 1));
    assert_eq!(isoweekdate_to_date((-1, 52, 5)), (-1, 12, 31));
    assert_eq!(isoweekdate_to_date((-1, 52, 6)), (0, 1, 1));
    assert_eq!(isoweekdate_to_date((-1, 52, 7)), (0, 1, 2));
    assert_eq!(isoweekdate_to_date((0, 1, 1)), (0, 1, 3));
    assert_eq!(isoweekdate_to_date((1970, 1, 4)), (1970, 1, 1));
    assert_eq!(isoweekdate_to_date((1976, 53, 6)), (1977, 1, 1));
    assert_eq!(isoweekdate_to_date((1976, 53, 7)), (1977, 1, 2));
    assert_eq!(isoweekdate_to_date((1977, 52, 6)), (1977, 12, 31));
    assert_eq!(isoweekdate_to_date((1977, 52, 7)), (1978, 1, 1));
    assert_eq!(isoweekdate_to_date((1978, 1, 1)), (1978, 1, 2));
    assert_eq!(isoweekdate_to_date((1978, 52, 7)), (1978, 12, 31));
    assert_eq!(isoweekdate_to_date((1979, 1, 1)), (1979, 1, 1));
    assert_eq!(isoweekdate_to_date((1979, 52, 7)), (1979, 12, 30));
    assert_eq!(isoweekdate_to_date((1980, 1, 1)), (1979, 12, 31));
    assert_eq!(isoweekdate_to_date((1980, 1, 2)), (1980, 1, 1));
    assert_eq!(isoweekdate_to_date((1980, 52, 7)), (1980, 12, 28));
    assert_eq!(isoweekdate_to_date((1981, 1, 1)), (1980, 12, 29));
    assert_eq!(isoweekdate_to_date((1981, 1, 2)), (1980, 12, 30));
    assert_eq!(isoweekdate_to_date((1981, 1, 3)), (1980, 12, 31));
    assert_eq!(isoweekdate_to_date((1981, 1, 4)), (1981, 1, 1));
    assert_eq!(isoweekdate_to_date((1981, 53, 4)), (1981, 12, 31));
    assert_eq!(isoweekdate_to_date((1981, 53, 5)), (1982, 1, 1));
    assert_eq!(isoweekdate_to_date((1981, 53, 6)), (1982, 1, 2));
}

#[test]
fn test_isoweeks_in_year() {
    assert_eq!(isoweeks_in_year(-3), 52); // wednesday
    assert_eq!(isoweeks_in_year(-2), 53); // thursday
    assert_eq!(isoweeks_in_year(-1), 52); // friday
    assert_eq!(isoweeks_in_year(0), 52); // saturday, leap year
    assert_eq!(isoweeks_in_year(1), 52); // monday
    assert_eq!(isoweeks_in_year(2), 52); // tuesday
    assert_eq!(isoweeks_in_year(3), 52); // wednesday
    assert_eq!(isoweeks_in_year(4), 53); // thursday, leap year
    assert_eq!(isoweeks_in_year(5), 52); // saturday
    assert_eq!(isoweeks_in_year(1969), 52); // wednesday
    assert_eq!(isoweeks_in_year(1970), 53); // thursday
    assert_eq!(isoweeks_in_year(1971), 52); // friday
    assert_eq!(isoweeks_in_year(2004), 53); // leap year, thursday
    assert_eq!(isoweeks_in_year(2015), 53); // thursday
    assert_eq!(isoweeks_in_year(2020), 53); // leap year, wednesday
}

#[test]
fn test_systemtime_to_secs() {
    assert_eq!(systemtime_to_secs(UNIX_EPOCH), Some((0, 0)));
    assert_eq!(
        systemtime_to_secs(UNIX_EPOCH + Duration::from_secs(RD_SECONDS_MAX as u64)),
        Some((RD_SECONDS_MAX, 0))
    );
    assert_eq!(
        systemtime_to_secs(UNIX_EPOCH - Duration::from_secs(-RD_SECONDS_MIN as u64)),
        Some((RD_SECONDS_MIN, 0))
    );
    assert_eq!(
        systemtime_to_secs(UNIX_EPOCH + Duration::from_secs(RD_SECONDS_MAX as u64 + 1)),
        None
    );
    assert_eq!(
        systemtime_to_secs(UNIX_EPOCH - Duration::from_secs(-RD_SECONDS_MIN as u64 + 1)),
        None
    );
}

#[test]
fn test_secs_to_systemtime() {
    assert_eq!(secs_to_systemtime((0, 0)), Some(UNIX_EPOCH));
    assert_eq!(
        secs_to_systemtime((RD_SECONDS_MAX, 0)),
        UNIX_EPOCH.checked_add(Duration::from_secs(RD_SECONDS_MAX as u64))
    );
    assert_eq!(
        secs_to_systemtime((RD_SECONDS_MIN, 0)),
        UNIX_EPOCH.checked_sub(Duration::from_secs(-RD_SECONDS_MIN as u64))
    );
}

#[test]
fn test_systemtime_to_datetime() {
    assert_eq!(systemtime_to_datetime(UNIX_EPOCH), Some((1970, 1, 1, 0, 0, 0, 0)));
    assert_eq!(
        systemtime_to_datetime(UNIX_EPOCH + Duration::from_secs(RD_SECONDS_MAX as u64)),
        Some((YEAR_MAX, 12, 31, 23, 59, 59, 0))
    );
    assert_eq!(
        systemtime_to_datetime(UNIX_EPOCH - Duration::from_secs(-RD_SECONDS_MIN as u64)),
        Some((YEAR_MIN, 1, 1, 0, 0, 0, 0))
    );
    assert_eq!(
        systemtime_to_datetime(UNIX_EPOCH + Duration::from_secs(RD_SECONDS_MAX as u64 + 1)),
        None
    );
    assert_eq!(
        systemtime_to_datetime(UNIX_EPOCH - Duration::from_secs(-RD_SECONDS_MIN as u64 + 1)),
        None
    );
}

#[test]
fn test_datetime_to_systemtime() {
    assert_eq!(datetime_to_systemtime((1970, 1, 1, 0, 0, 0, 0)), Some(UNIX_EPOCH));
    assert_eq!(
        datetime_to_systemtime((YEAR_MAX, 12, 31, 23, 59, 59, 0)),
        UNIX_EPOCH.checked_add(Duration::from_secs(RD_SECONDS_MAX as u64))
    );
    assert_eq!(
        datetime_to_systemtime((YEAR_MIN, 1, 1, 0, 0, 0, 0)),
        UNIX_EPOCH.checked_sub(Duration::from_secs(-RD_SECONDS_MIN as u64))
    );
}
