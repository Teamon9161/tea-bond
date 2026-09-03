//! 通过本机 Wind 终端下载债券信息（Windows / Linux）。
//!
//! 相比爬取公开网页，Wind 的字段更全，且银行间和交易所走同一套接口，
//! 发行价（`issue_issueprice`）也只有这条路径能拿到。

use crate::SmallStr;
use crate::bond::{Bond, BondDayCount, CouponType, InterestType, Market};
use anyhow::{Result, anyhow, bail};
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;
use wind_rs::{Data, Wind};

const WIND_FIELDS: &str = "sec_name,carrydate,maturitydate,interesttype,couponrate,\
     actualbenchmark,coupon,interestfrequency,latestpar,issue_issueprice";

/// 单次 wss 请求的代码数上限
const WSS_CHUNK: usize = 1000;

type Job = Box<dyn FnOnce(&mut Option<Wind>) + Send>;

/// Wind 的底层库是进程级单例，Windows 的 COM 组件还要求在创建它的线程上调用，
/// 所以会话固定住在一个专属线程里，所有请求排队送过去。
fn worker() -> &'static Sender<Job> {
    static WORKER: OnceLock<Sender<Job>> = OnceLock::new();
    WORKER.get_or_init(|| {
        let (tx, rx) = channel::<Job>();
        std::thread::Builder::new()
            .name("tea-bond-wind".into())
            .spawn(move || {
                let mut session: Option<Wind> = None;
                while let Ok(job) = rx.recv() {
                    job(&mut session);
                }
            })
            .expect("Failed to spawn wind worker thread");
        tx
    })
}

/// 连接超时，默认 20 秒（WindPy 默认 120 秒，对下载来说太久）
fn connect_timeout() -> Duration {
    let secs = std::env::var("WIND_CONNECT_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    Duration::from_secs(secs)
}

/// 在 Wind 会话线程上执行一次请求，会话按需建立并复用
fn with_wind<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&Wind) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = channel();
    worker()
        .send(Box::new(move |session: &mut Option<Wind>| {
            let res = (|| -> Result<T> {
                if session.is_none() {
                    *session = Some(Wind::builder().timeout(connect_timeout()).connect()?);
                }
                f(session.as_ref().unwrap())
            })();
            // 终端退出或断线时丢弃会话，下次请求重连
            if res.is_err() && session.as_ref().is_some_and(|w| !w.is_connected()) {
                *session = None;
            }
            let _ = tx.send(res);
        }))
        .map_err(|_| anyhow!("Wind worker thread is down"))?;
    rx.recv()
        .map_err(|_| anyhow!("Wind worker thread panicked"))?
}

/// Wind 代码的市场后缀
pub(super) fn market_suffix(market: Market) -> &'static str {
    match market {
        Market::IB => "IB",
        Market::SH | Market::SSE => "SH",
        Market::SZ | Market::SZE => "SZ",
    }
}

fn get_coupon_type(typ: &str) -> Result<CouponType> {
    match typ {
        "附息" => Ok(CouponType::CouponBear),
        "到期一次还本付息" => Ok(CouponType::OneTime),
        "贴现" => Ok(CouponType::ZeroCoupon),
        typ => bail!("Cannot infer coupon type from Wind: {}", typ),
    }
}

fn get_interest_type(typ: &str) -> Result<InterestType> {
    match typ {
        "固定利率" => Ok(InterestType::Fixed),
        "浮动利率" => Ok(InterestType::Floating),
        "累进利率" => Ok(InterestType::Progressive),
        "零息" => Ok(InterestType::Zero),
        typ => bail!("Cannot infer interest type from Wind: {}", typ),
    }
}

#[inline]
fn round(f: f64, precision: i32) -> f64 {
    let factor = 10f64.powi(precision);
    (f * factor).round() / factor
}

fn parse_bond(data: &Data, code: &str) -> Result<Bond> {
    // Wind 对查不到的代码不报错，整行返回空值
    let get = |field: &str| data.at(0, code, field).filter(|v| !v.is_empty());
    let field = |name: &'static str| {
        get(name).ok_or_else(|| anyhow!("Wind returns no {} for bond {}", name, code))
    };

    let cp_type = get_coupon_type(field("coupon")?.as_str().unwrap_or_default())?;
    // Wind 把贴现债的利率类型也标成"固定利率"，与 china_money / wind_sql 的口径对齐成零息
    let interest_type = if let CouponType::ZeroCoupon = cp_type {
        InterestType::Zero
    } else {
        get_interest_type(field("interesttype")?.as_str().unwrap_or_default())?
    };
    let (base_rate, rate_spread) = if let InterestType::Floating = interest_type {
        bail!("Get base rate & rate spread for floating bond from Wind is not implemented yet");
    } else {
        (None, None)
    };
    let inst_freq = match cp_type {
        CouponType::CouponBear => field("interestfrequency")?
            .as_i64()
            .ok_or_else(|| anyhow!("Invalid interest frequency for bond {}", code))?
            as i32,
        CouponType::OneTime => 1,
        CouponType::ZeroCoupon => 0,
    };
    let day_count = get("actualbenchmark")
        .and_then(|v| v.as_str())
        .map(|s| {
            BondDayCount::from_str(s).unwrap_or_else(|_| {
                eprintln!("Unknown day count {s} for bond {code}, use default");
                BondDayCount::default()
            })
        })
        .unwrap_or_default();
    let market = code
        .split_once('.')
        .and_then(|(_, m)| m.parse::<Market>().ok())
        .unwrap_or_default();

    Ok(Bond {
        bond_code: SmallStr::new(code),
        mkt: market,
        abbr: field("sec_name")?.to_string().into(),
        par_value: get("latestpar").and_then(|v| v.as_f64()).unwrap_or(100.),
        cp_type,
        interest_type,
        // Wind 的票面利率是百分数
        cp_rate: round(
            field("couponrate")?
                .as_f64()
                .ok_or_else(|| anyhow!("Invalid coupon rate for bond {}", code))?
                * 0.01,
            8,
        ),
        base_rate,
        rate_spread,
        inst_freq,
        carry_date: field("carrydate")?
            .as_date()
            .ok_or_else(|| anyhow!("Invalid carry date for bond {}", code))?,
        maturity_date: field("maturitydate")?
            .as_date()
            .ok_or_else(|| anyhow!("Invalid maturity date for bond {}", code))?,
        day_count,
        issue_price: get("issue_issueprice").and_then(|v| v.as_f64()),
    })
}

impl Bond {
    /// 从本机 Wind 终端下载单只债券的信息
    ///
    /// `code` 需带市场后缀，如 `240012.IB` / `019733.SH`
    pub fn wind_download(code: &str) -> Result<Bond> {
        let mut bonds = Self::wind_download_batch(&[code])?;
        bonds.pop().unwrap()
    }

    /// 批量下载多只债券的信息，逐只返回结果
    ///
    /// wss 支持多代码多字段，一次请求就能取回整批（不是逐只循环）。
    /// 实测单次请求上限在 5000~10000 只之间，超限会直接报错而不是静默截断，
    /// 所以按 [`WSS_CHUNK`] 分批，留足余量。
    pub fn wind_download_batch(codes: &[&str]) -> Result<Vec<Result<Bond>>> {
        let options = format!(
            "tradeDate={}",
            chrono::Local::now().date_naive().format("%Y%m%d")
        );
        let mut bonds = Vec::with_capacity(codes.len());
        for chunk in codes.chunks(WSS_CHUNK) {
            let codes: Vec<String> = chunk.iter().map(|c| c.to_string()).collect();
            let query = codes.clone();
            let options = options.clone();
            let data = with_wind(move |wind| Ok(wind.wss(query, WIND_FIELDS, options)?))?;
            bonds.extend(codes.iter().map(|code| parse_bond(&data, code)));
        }
        Ok(bonds)
    }
}
