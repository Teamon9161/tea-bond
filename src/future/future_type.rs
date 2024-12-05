use chrono::{Datelike, NaiveDate};

#[derive(Debug, Copy, Clone)]
pub enum FutureType {
    TS, // 2年期国债期货
    TF, // 5年期国债期货
    T,  // 10年期国债期货
    TL, // 30年期国债期货
}

#[inline]
fn issue_year(maturity_date: NaiveDate, carry_date: NaiveDate) -> i32 {
    maturity_date.year() - carry_date.year()
}

#[inline]
fn remain_year(delivery_date: NaiveDate, maturity_date: NaiveDate) -> f64 {
    let year_diff = maturity_date.year() - delivery_date.year();
    let month_diff = maturity_date.month() as i32 - delivery_date.month() as i32;
    let date_diff = maturity_date.day() as i32 - 1; // 规定为于期货到期月首日的差值
    year_diff as f64 + month_diff as f64 / 12.0 + date_diff as f64 / 365.0
}

impl FutureType {
    /// 判断是不是可交割券
    pub fn is_deliverable(
        &self,
        delivery_date: NaiveDate,
        carry_date: NaiveDate,
        maturity_date: NaiveDate,
    ) -> bool {
        let issue_year = issue_year(maturity_date, carry_date);
        let remain_year = remain_year(delivery_date, maturity_date);
        match self {
            // 2年期国债期货
            // 发行期限不高于5年，合约到期月份首日剩余期限为1.5-2.25年的记账式附息国债
            FutureType::TS => issue_year <= 5 && (1.5..=2.25).contains(&remain_year),
            // 5年期国债期货
            // 发行期限不高于7年、合约到期月份首日剩余期限为4-5.25年的记账式附息国债
            FutureType::TF => issue_year <= 7 && (4.0..=5.25).contains(&remain_year),
            // 10年期国债期货
            // 发行期限不高于10年、合约到期月份首日剩余期限不低于6.5年的记账式附息国债
            FutureType::T => issue_year <= 10 && remain_year >= 6.5,
            // 30年期国债期货
            // 发行期限不高于30年，合约到期月份首日剩余期限不低于25年的记账式附息国债
            FutureType::TL => issue_year <= 30 && remain_year >= 25.0,
        }
    }
}
