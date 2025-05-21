pub mod china;

// pub use china::*;
use chrono::NaiveDate;

macro_rules! date {
    ($year:expr, $month:expr, $day:expr) => {
        ::chrono::NaiveDate::from_ymd_opt($year, $month, $day).unwrap()
    };
}
pub(crate) use date;

pub trait Calendar {
    fn is_business_day(&self, date: NaiveDate) -> bool;

    fn find_workday(&self, mut date: NaiveDate, mut offset: i32) -> NaiveDate {
        if offset >= 0 {
            while offset >= 0 {
                if self.is_business_day(date) {
                    offset -= 1
                }
                if offset >= 0 {
                    date = date.succ_opt().unwrap();
                }
            }
            date
        } else {
            while offset < 0 {
                date = date.pred_opt().unwrap();
                if self.is_business_day(date) {
                    offset += 1;
                }
            }
            date
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Calendar;
    use super::china::*;
    #[test]
    fn test_find_workday() {
        assert_eq!(SSE.find_workday(date!(2025, 5, 16), 0), date!(2025, 5, 16));
        assert_eq!(SSE.find_workday(date!(2025, 5, 16), 1), date!(2025, 5, 19));
        assert_eq!(SSE.find_workday(date!(2025, 5, 17), 0), date!(2025, 5, 19));
        assert_eq!(SSE.find_workday(date!(2025, 5, 16), 8), date!(2025, 5, 28));
        assert_eq!(SSE.find_workday(date!(2025, 5, 18), -1), date!(2025, 5, 16));
        assert_eq!(SSE.find_workday(date!(2025, 5, 6), -1), date!(2025, 4, 30));
        assert_eq!(SSE.find_workday(date!(2025, 5, 6), -2), date!(2025, 4, 29));
        assert_eq!(SSE.find_workday(date!(2025, 5, 6), -8), date!(2025, 4, 21));
        assert_eq!(IB.find_workday(date!(2025, 5, 6), -8), date!(2025, 4, 22));
    }
}
