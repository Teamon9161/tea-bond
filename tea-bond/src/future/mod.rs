mod future_price;
mod future_type;
mod impls;

pub use future_price::FuturePrice;
pub use future_type::FutureType;

use crate::SmallStr;
use anyhow::{Result, anyhow, bail};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use tea_calendar::{Calendar, china::CFFEX};

const CFFEX_DEFAULT_CP_RATE: f64 = 0.03;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Future {
    pub code: SmallStr,
    pub market: Option<SmallStr>,
}

impl Default for Future {
    #[inline]
    fn default() -> Self {
        Self {
            code: "T2412".into(),
            market: None,
        }
    }
}

impl Future {
    #[inline]
    pub fn new(code: impl AsRef<str>) -> Self {
        let code = code.as_ref();
        if let Some((code, market)) = code.split_once('.') {
            Self {
                code: code.into(),
                market: Some(market.into()),
            }
        } else {
            Self {
                code: code.into(),
                market: None,
            }
        }
    }

    #[inline]
    /// 获取下一季月合约
    pub fn next_future(&self) -> Result<Self> {
        self.shift_by_quarter(3)
    }

    #[inline]
    /// 获取上一季月合约
    pub fn prev_future(&self) -> Result<Self> {
        self.shift_by_quarter(-3)
    }

    #[inline]
    pub fn future_type(&self) -> Result<FutureType> {
        let typ = self.code.replace(|c: char| c.is_numeric(), "");
        typ.parse()
    }

    #[inline]
    /// 判断是否是可交割券
    ///
    /// delivery_date: 可以传入已计算过的期货配对缴款日避免重复计算
    pub fn is_deliverable(
        &self,
        carry_date: NaiveDate,
        maturity_date: NaiveDate,
        delivery_date: Option<NaiveDate>,
    ) -> Result<bool> {
        Ok(self.future_type()?.is_deliverable(
            delivery_date.unwrap_or_else(|| self.deliver_date().unwrap()),
            carry_date,
            maturity_date,
        ))
    }

    /// 计算期货合约的最后交易日
    ///
    /// 计算国债期货的最后交易日=合约到期月份的第二个星期五
    /// 根据合约代码, 依据中金所的国债期货合约最后交易日的说, 返回该合约的最后交易日
    /// 获取年月部分
    pub fn last_trading_date(&self) -> Result<NaiveDate> {
        let yymm = self.code.replace(|c: char| c.is_alphabetic(), "");
        let yyyy = if let Some(yy) = yymm.get(0..2) {
            format!("20{yy}")
        } else {
            bail!("Can not extract year from future code: {}", self.code);
        };
        let mm = if let Some(mm) = yymm.get(2..) {
            mm
        } else {
            bail!("Can not extract month from future code: {}", self.code);
        };
        // 构造交割月的第一天
        let begin_day_of_month = NaiveDate::from_ymd_opt(yyyy.parse()?, mm.parse()?, 1).unwrap();
        // 第2个周五,月初首日的第0-6天不需要计算
        for i in 7..14 {
            let date_i = begin_day_of_month + Duration::days(i);
            if let Weekday::Fri = date_i.weekday() {
                return Ok(date_i);
            }
        }
        bail!("No valid trading date found")
    }

    /// 获取期货合约的配对缴款日
    ///
    /// 交割日为3天,其中第2天为缴款日,即最后交易日的第2个交易日,最后交易日一定为周五,所以缴款日一定是一个周二
    #[inline]
    pub fn deliver_date(&self) -> Result<NaiveDate> {
        let last_trading_date = self.last_trading_date()?;
        Ok(CFFEX.find_workday(last_trading_date, 2))
        // Ok(last_trading_date + Duration::days(4))
    }

    /// 获取期货合约的首个交易日
    ///
    /// 对于首批上市合约,返回该品种的上市日期;
    /// 对于后续合约,返回前3季度合约最后交易日的下一个交易日
    pub fn first_trading_date(&self) -> Result<NaiveDate> {
        let typ = self.future_type()?;
        let offset = quarters_since_first(self, typ)?;
        if offset < 3 {
            Ok(typ.listing_start_date())
        } else {
            let prev = self.shift_by_quarter(-9)?;
            Ok(CFFEX.find_workday(prev.last_trading_date()?, 1))
        }
    }

    /// 获取期货合约的交易区间
    ///
    /// 返回 (首个交易日, 最后交易日)
    #[inline]
    pub fn trading_window(&self) -> Result<(NaiveDate, NaiveDate)> {
        Ok((self.first_trading_date()?, self.last_trading_date()?))
    }

    /// 获取指定时间段内该类型期货的所有交易合约
    ///
    /// 每个合约的交易区间为「上一个(前3个季度)合约最后交易日的下一个交易日」到「本合约最后交易日」。
    /// 缺省 end 使用 start。
    pub fn trading_futures(
        start: NaiveDate,
        end: Option<NaiveDate>,
        future_type: Option<FutureType>,
    ) -> Result<Vec<Self>> {
        let end = end.unwrap_or(start);
        if end < start {
            bail!("end date should not be earlier than start date");
        }

        let types: Vec<FutureType> = if let Some(t) = future_type {
            vec![t]
        } else {
            vec![
                FutureType::TS,
                FutureType::TF,
                FutureType::T,
                FutureType::TL,
            ]
        };

        let mut res = Vec::new();
        for t in types {
            res.extend(trading_futures_by_type(t, start, end)?);
        }

        Ok(res)
    }

    #[inline]
    fn shift_by_quarter(&self, delta_months: i32) -> Result<Self> {
        let code = self.code.as_str();
        let (prefix, year, month) = parse_code(code)?;

        let month_index = year as i32 * 12 + month as i32 - 1 + delta_months;
        if month_index < 0 {
            bail!("Future month underflow: {}", code);
        }

        let new_year = (month_index / 12) as u32;
        let new_month = (month_index % 12 + 1) as u32;

        Ok(Self {
            code: format!("{prefix}{new_year:02}{new_month:02}").into(),
            market: self.market.clone(),
        })
    }
}

fn trading_futures_by_type(
    typ: FutureType,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<Future>> {
    let mut result = Vec::new();

    let listing_start = typ.listing_start_date();
    if end < listing_start {
        return Ok(result);
    }

    let mut future = future_from_date(typ, start.max(listing_start));
    while quarters_since_first(&future, typ)? < 0 {
        future = future.next_future()?;
    }

    loop {
        let (s, _) = future.trading_window()?;
        if s <= start || quarters_since_first(&future, typ)? == 0 {
            break;
        }
        future = future.prev_future()?;
    }

    loop {
        let (s, e) = future.trading_window()?;
        if s > end {
            break;
        }

        if e >= start {
            result.push(future.clone());
        }

        future = future.next_future()?;
    }

    Ok(result)
}

fn future_from_date(typ: FutureType, date: NaiveDate) -> Future {
    let q_month = ((date.month() - 1) / 3) * 3 + 3;
    let year = (date.year() % 100) as u32;
    Future::new(format!("{}{:02}{q_month:02}", typ.prefix(), year))
}

fn parse_code(code: &str) -> Result<(&str, u32, u32)> {
    let (prefix, digits) = code
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(idx, _)| code.split_at(idx))
        .ok_or_else(|| anyhow!("Invalid future code: {}", code))?;

    if digits.len() != 4 {
        bail!("Invalid future code: {}", code);
    }

    let yymm: u32 = digits.parse()?;
    let year = yymm / 100;
    let month = yymm % 100;

    if !(1..=12).contains(&month) {
        bail!("Invalid future month: {}", code);
    }

    Ok((prefix, year, month))
}

fn quarters_since_first(future: &Future, typ: FutureType) -> Result<i32> {
    let (_, year, month) = parse_code(future.code.as_str())?;
    let first_code = typ.first_contracts()[0];
    let (_, base_year, base_month) = parse_code(first_code)?;
    let month_index = year as i32 * 12 + month as i32;
    let base_index = base_year as i32 * 12 + base_month as i32;
    let diff = month_index - base_index;
    if diff % 3 != 0 {
        bail!("Invalid future code month offset: {}", future.code);
    }
    Ok(diff / 3)
}

/// [中金所转换因子计算公式](http://www.cffex.com.cn/10tf/)
///
/// r：10/5/2年期国债期货合约票面利率3%；
/// x：交割月到下一付息月的月份数；
/// n：剩余付息次数；
/// c：可交割国债的票面利率；
/// f：可交割国债每年的付息次数；
/// 计算结果四舍五入至小数点后4位。
fn cffex_tb_cf_formula(n: i32, c: f64, f: f64, x: i32, r: Option<f64>) -> f64 {
    let r = r.unwrap_or(CFFEX_DEFAULT_CP_RATE);
    let cf = (c / f + c / r + (1.0 - c / r) / (1.0 + r / f).powi(n - 1))
        / (1.0 + r / f).powf(x as f64 * f / 12.0)
        - (1.0 - x as f64 * f / 12.0) * c / f;
    (cf * 10000.0).round() / 10000.0
}

/// 根据中金所公式计算转换因子
///
/// remaining_cp_times_after_dlv:交割券剩余付息次数,缴款日之后
///
/// cp_rate:交割券的票面利率
///
/// inst_freq:交割券的年付息次数
///
/// month_number_to_next_cp_after_dlv:交割月到下个付息日之间的月份数
///
/// fictitious_cp_rate:虚拟券票面利率,默认值为3%
#[inline]
pub fn calc_cf(
    remaining_cp_times_after_dlv: i32,
    cp_rate: f64,
    inst_freq: i32,
    month_number_to_next_cp_after_dlv: i32,
    fictitious_cp_rate: Option<f64>,
) -> f64 {
    cffex_tb_cf_formula(
        remaining_cp_times_after_dlv,
        cp_rate,
        inst_freq as f64,
        month_number_to_next_cp_after_dlv,
        fictitious_cp_rate,
    )
}

#[cfg(test)]
mod tests {
    use super::{Future, FutureType};
    use chrono::NaiveDate;

    #[test]
    fn shift_across_year() {
        let f = Future::new("T2412");
        assert_eq!(f.next_future().unwrap().code.as_str(), "T2503");
        let f = Future::new("T2503");
        assert_eq!(f.prev_future().unwrap().code.as_str(), "T2412");
        let f = Future::new("T0812");
        assert_eq!(f.next_future().unwrap().code.as_str(), "T0903");
        assert_eq!(f.prev_future().unwrap().code.as_str(), "T0809");
    }

    #[test]
    fn keep_market_suffix() {
        let f = Future::new("TF2409.CFE");
        let next = f.next_future().unwrap();
        assert_eq!(next.code.as_str(), "TF2412");
        assert_eq!(next.market.as_deref(), Some("CFE"));
    }

    #[test]
    fn tl_quarterly_contract() {
        let f = Future::new("TL2506");
        assert_eq!(f.next_future().unwrap().code.as_str(), "TL2509");
    }

    #[test]
    fn first_trading_date() {
        // 首批合约返回上市日期
        let f = Future::new("T1509");
        assert_eq!(
            f.first_trading_date().unwrap(),
            NaiveDate::from_ymd_opt(2015, 3, 20).unwrap()
        );
        let f = Future::new("TF1312");
        assert_eq!(
            f.first_trading_date().unwrap(),
            NaiveDate::from_ymd_opt(2013, 9, 6).unwrap()
        );
        // 后续合约返回前9个月合约最后交易日的下一个交易日
        // T2509的前9个月合约是T2412, 其最后交易日是2024-12-13(周五), 下一交易日是2024-12-16(周一)
        let f = Future::new("T2509");
        assert_eq!(
            f.first_trading_date().unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 16).unwrap()
        );
    }

    #[test]
    fn trading_futures_in_range() {
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2025, 9, 12).unwrap(),
            None,
            Some(FutureType::T),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("T2509"),
                Future::new("T2512"),
                Future::new("T2603")
            ]
        );
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(),
            None,
            Some(FutureType::T),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("T2512"),
                Future::new("T2603"),
                Future::new("T2606")
            ]
        );
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2025, 9, 30).unwrap()),
            Some(FutureType::T),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("T2509"),
                Future::new("T2512"),
                Future::new("T2603"),
                Future::new("T2606")
            ]
        );
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2013, 9, 10).unwrap(),
            None,
            Some(FutureType::TF),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("TF1312"),
                Future::new("TF1403"),
                Future::new("TF1406")
            ]
        );
    }

    #[test]
    fn trading_futures_listing_starts() {
        // 10Y first day
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2015, 3, 20).unwrap(),
            None,
            Some(FutureType::T),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("T1509"),
                Future::new("T1512"),
                Future::new("T1603")
            ]
        );

        // 5Y first day
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2013, 9, 6).unwrap(),
            None,
            Some(FutureType::TF),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("TF1312"),
                Future::new("TF1403"),
                Future::new("TF1406")
            ]
        );

        // 2Y first day
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2018, 8, 17).unwrap(),
            None,
            Some(FutureType::TS),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("TS1812"),
                Future::new("TS1903"),
                Future::new("TS1906")
            ]
        );

        // 30Y first day
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2023, 4, 21).unwrap(),
            None,
            Some(FutureType::TL),
        )
        .unwrap();
        assert_eq!(
            futures,
            vec![
                Future::new("TL2306"),
                Future::new("TL2309"),
                Future::new("TL2312")
            ]
        );

        // before listing should be empty
        let futures = Future::trading_futures(
            NaiveDate::from_ymd_opt(2023, 4, 10).unwrap(),
            None,
            Some(FutureType::TL),
        )
        .unwrap();
        assert!(futures.is_empty());
    }
}
