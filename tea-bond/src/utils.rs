use chrono::{Datelike, NaiveDate};

#[inline]
pub fn month_delta(from_date: NaiveDate, to_date: NaiveDate) -> i32 {
    let from_date_month = from_date.month();
    let to_date_month = to_date.month();
    let from_date_year = from_date.year();
    let to_date_year = to_date.year();
    (to_date_year - from_date_year) * 12 + (to_date_month as i32 - from_date_month as i32)
}

pub fn bisection_find_ytm<F>(f: F, lower: f64, upper: f64, degree: Option<i32>) -> f64
where
    F: Fn(f64) -> f64,
{
    let epsilon = 10f64.powi(-degree.unwrap_or(15));
    assert!(upper > lower);
    let mut lower = lower;
    let mut upper = upper;
    let f_lower = f(lower);
    let f_upper = f(upper);
    let sign = f_upper >= f_lower;
    if sign {
        while upper - lower > epsilon {
            let mid = (lower + upper) / 2.0;
            let f_mid = f(mid);
            if f_mid == 0.0 {
                return mid;
            } else if f_mid < 0.0 {
                lower = mid;
            } else {
                upper = mid;
            }
        }
    } else {
        while upper - lower > epsilon {
            let mid = (lower + upper) / 2.0;
            let f_mid = f(mid);
            if f_mid == 0.0 {
                return mid;
            } else if f_mid < 0.0 {
                upper = mid;
            } else {
                lower = mid;
            }
        }
    }
    (lower + upper) * 0.5
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_bisection_find_ytm() {
        // positive sign
        let f = |x: f64| x.powi(2) - 2.0;
        let ytm = bisection_find_ytm(f, 0.0, 2.0, None);
        assert!((ytm - 1.41421356237).abs() <= 1e-10);
        // negative sign
        let f = |x: f64| -x.powi(2) + 2.0;
        let ytm = bisection_find_ytm(f, 0.0, 2.0, None);
        assert!((ytm - 1.41421356237).abs() <= 1e-10);
    }
}
