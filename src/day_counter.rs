use chrono::{Datelike, NaiveDate};

/// Represents the Actual/Actual day count convention.
pub const ACTUAL: Actual = Actual {};

/// Represents the Thirty/360 day count convention.
pub const THIRTY: Thirty = Thirty {};

/// Represents the Business day count convention.
pub const BUSINESS: Business = Business {};

/// 计算两个日期之间的天数，不同规则有不同实现
pub trait DayCountRule {
    fn count_days(&self, start: NaiveDate, end: NaiveDate) -> i64;
}

/// Represents the Actual day count convention.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Actual {}

impl DayCountRule for Actual {
    /// Calculates the number of days between two dates using the Actual day count convention.
    #[inline]
    fn count_days(&self, start: NaiveDate, end: NaiveDate) -> i64 {
        (end - start).num_days()
    }
}

/// Represents the Thirty/360 day count convention.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Thirty {}

impl DayCountRule for Thirty {
    /// Calculates the number of days between two dates using the Thirty/360 day count convention.
    /// Assumes each month has 30 days and a year has 360 days.
    #[inline]
    fn count_days(&self, start: NaiveDate, end: NaiveDate) -> i64 {
        360 * (end.year() as i64 - start.year() as i64)
            + 30 * (end.month() as i64 - start.month() as i64)
            + (end.day() as i64 - start.day() as i64)
    }
}

/// Represents the Business day count convention.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Business {}

impl DayCountRule for Business {
    /// Calculates the number of business days (Monday to Friday) between two dates.
    /// Excludes weekends (Saturday and Sunday).
    #[inline]
    fn count_days(&self, start: NaiveDate, end: NaiveDate) -> i64 {
        let total_days = (end - start).num_days();
        let full_weeks = total_days / 7;
        let remaining_days = total_days % 7;

        let mut business_days = full_weeks * 5; // 5 working days per week

        let start_weekday = start.weekday().num_days_from_monday();

        for i in 0..remaining_days {
            let current_weekday = (start_weekday + i as u32 + 1) % 7;
            if current_weekday < 5 {
                business_days += 1;
            }
        }

        business_days
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_actual_count_days() {
        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        assert_eq!(ACTUAL.count_days(start, end), 9);
    }

    #[test]
    fn test_thirty_count_days() {
        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        assert_eq!(THIRTY.count_days(start, end), 9);

        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 2, 3).unwrap();
        assert_eq!(THIRTY.count_days(start, end), 32);

        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 4).unwrap();
        assert_eq!(THIRTY.count_days(start, end), 423);
    }

    #[test]
    fn test_business_count_days() {
        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
        assert_eq!(BUSINESS.count_days(start, end), 7);

        let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 1, 15).unwrap();
        assert_eq!(BUSINESS.count_days(start, end), 10);

        let start = NaiveDate::from_ymd_opt(2023, 1, 4).unwrap();
        let end = NaiveDate::from_ymd_opt(2023, 3, 20).unwrap();
        assert_eq!(BUSINESS.count_days(start, end), 53);
    }
}
