mod future_price;
mod future_type;
mod impls;

pub use future_price::FuturePrice;
pub use future_type::FutureType;

use crate::SmallStr;
use anyhow::{bail, Result};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use tea_calendar::{china::CFFEX, Calendar};

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

    #[inline]
    pub fn future_type(&self) -> Result<FutureType> {
        let typ = self.code.replace(|c: char| c.is_numeric(), "");
        typ.parse()
    }
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
