use chrono::{Datelike, NaiveDate};

#[inline]
pub fn month_delta(from_date: NaiveDate, to_date: NaiveDate) -> i32 {
    let from_date_month = from_date.month();
    let to_date_month = to_date.month();
    let from_date_year = from_date.year();
    let to_date_year = to_date.year();
    let month_delta =
        (to_date_year - from_date_year) * 12 + (to_date_month as i32 - from_date_month as i32);
    month_delta
}
