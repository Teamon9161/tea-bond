use anyhow::{bail, Result};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::sync::Arc;

const CFFEX_DEFAULT_CP_RATE: f64 = 0.03;

#[derive(Debug, Clone)]
pub struct Future {
    pub code: Arc<str>,
}

impl Default for Future {
    fn default() -> Self {
        Self {
            code: Arc::from("T2412"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FutureType {
    TS,
    TF,
    T,
    TL,
}

impl Future {
    #[inline]
    pub fn new(code: impl AsRef<str>) -> Self {
        Self {
            code: Arc::from(code.as_ref()),
        }
    }

    /// 计算期货合约的最后交易日
    ///
    /// 计算国债期货的最后交易日=合约到期月份的第二个星期五
    /// 根据合约代码, 依据中金所的国债期货合约最后交易日的说, 返回该合约的最后交易日
    /// 获取年月部分
    pub fn last_trading_date(&self) -> Result<NaiveDate> {
        let yymm = self.code.replace(|c: char| c.is_alphabetic(), "");
        let yyyy = format!("20{}", &yymm[0..2]);
        let mm = &yymm[2..];
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
    pub fn paydate(&self) -> Result<NaiveDate> {
        let last_trading_date = self.last_trading_date()?;
        Ok(last_trading_date + Duration::days(4))
    }

    #[inline]
    pub fn future_type(&self) -> Result<FutureType> {
        let code = self.code.as_ref();
        let typ = code.replace(|c: char| c.is_numeric(), "");
        match typ.as_str() {
            "T" => Ok(FutureType::T),
            "TF" => Ok(FutureType::TF),
            "TS" => Ok(FutureType::TS),
            "TL" => Ok(FutureType::TL),
            _ => bail!("Invalid future type: {}", typ),
        }
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
/// b_remaining_cp_times_after_dlv:交割券剩余付息次数,缴款日之后
///
/// b_cp_rate:交割券的票面利率
///
/// b_inst_freq:交割券的年付息次数
///
/// month_number_to_next_cp_after_dlv:交割月到下个付息日之间的月份数
///
/// fictitious_cp_rate:虚拟券票面利率,默认值为3%
pub fn calc_cf(
    b_remaining_cp_times_after_dlv: i32,
    b_cp_rate: f64,
    b_inst_freq: i32,
    month_number_to_next_cp_after_dlv: i32,
    fictitious_cp_rate: Option<f64>,
) -> f64 {
    cffex_tb_cf_formula(
        b_remaining_cp_times_after_dlv,
        b_cp_rate,
        b_inst_freq as f64,
        month_number_to_next_cp_after_dlv,
        fictitious_cp_rate,
    )
}
