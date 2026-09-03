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
    let (f_lower, f_upper) = (f(lower), f(upper));
    if f_lower.is_nan() || f_upper.is_nan() {
        // 输入数据缺失（如行情或债券信息缺失）导致目标函数无法求值，
        // 直接返回NaN，避免NaN参与比较时退化为恒真/恒假从而收敛到区间端点
        return f64::NAN;
    }
    let move_lower_on_negative = f_upper >= f_lower;

    while upper - lower > epsilon {
        let mid = (lower + upper) / 2.0;
        let f_mid = f(mid);

        if f_mid.is_nan() {
            return f64::NAN;
        }

        if f_mid == 0.0 {
            return mid;
        }

        if (f_mid < 0.0) == move_lower_on_negative {
            lower = mid;
        } else {
            upper = mid;
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

    #[test]
    fn test_bisection_find_ytm_nan_input() {
        // 模拟行情/债券信息缺失导致目标函数恒为NaN的情况，
        // 应返回NaN而不是收敛到区间端点
        let f = |x: f64| x.powi(2) - f64::NAN;
        let ytm = bisection_find_ytm(f, 0.0, 2.0, None);
        assert!(ytm.is_nan());
    }
}
