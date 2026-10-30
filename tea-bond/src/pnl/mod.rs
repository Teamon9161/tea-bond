mod fee;
mod trade_from_signal;

pub use fee::Fee;
use std::path::PathBuf;
pub use trade_from_signal::{QtyRoundMode, TradeFromPosOpt, trading_from_pos};

use crate::{Bond, CachedBond};
use chrono::NaiveDate;

use anyhow::{Result, anyhow};
use itertools::Either;
use itertools::izip;
use serde::Deserialize;
use tea_calendar::Calendar;
use tevec::prelude::{EPS, IsNone, Number, Vec1, Vec1View};

fn coupon_payment_count_between(
    bond: &Bond,
    from_exclusive: NaiveDate,
    to_exclusive: NaiveDate,
) -> Result<usize> {
    if from_exclusive >= to_exclusive {
        return Ok(0);
    }
    if bond.inst_freq <= 0 {
        return Ok(0);
    }
    let (mut cp_date, _) = bond.get_nearest_cp_date(from_exclusive)?;
    let offset = bond.get_cp_offset()?;
    let mut count = 0;
    while cp_date <= to_exclusive {
        let payment_date = bond.mkt.find_workday(cp_date, 0);
        if payment_date > from_exclusive && payment_date < to_exclusive {
            count += 1;
        }
        cp_date = cp_date + offset;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Market;
    use tea_calendar::Calendar;

    #[test]
    fn coupon_payment_count_uses_adjusted_workday_open_interval() {
        let bond = Bond {
            mkt: Market::IB,
            carry_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2031, 9, 15).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };

        let before_nominal = NaiveDate::from_ymd_opt(2025, 9, 12).unwrap();
        let payment_day = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap();
        let after_payment = NaiveDate::from_ymd_opt(2025, 9, 16).unwrap();

        assert_eq!(
            coupon_payment_count_between(&bond, before_nominal, payment_day).unwrap(),
            0
        );
        assert_eq!(
            coupon_payment_count_between(&bond, before_nominal, after_payment).unwrap(),
            1
        );
        assert_eq!(
            coupon_payment_count_between(&bond, payment_day, after_payment).unwrap(),
            0
        );
    }

    #[test]
    fn coupon_payment_count_checks_previous_nominal_coupon_date() {
        let bond = Bond {
            mkt: Market::IB,
            carry_date: NaiveDate::from_ymd_opt(2024, 5, 31).unwrap(),
            maturity_date: NaiveDate::from_ymd_opt(2031, 5, 31).unwrap(),
            inst_freq: 1,
            ..Default::default()
        };

        let nominal_coupon_date = NaiveDate::from_ymd_opt(2025, 5, 31).unwrap();
        let after_payment = NaiveDate::from_ymd_opt(2025, 6, 4).unwrap();

        assert_eq!(
            bond.mkt.find_workday(nominal_coupon_date, 0),
            NaiveDate::from_ymd_opt(2025, 6, 3).unwrap()
        );
        assert_eq!(
            coupon_payment_count_between(&bond, nominal_coupon_date, after_payment).unwrap(),
            1
        );
    }

    #[test]
    fn avg_price_tracks_open_cost_through_reduce_reverse_and_reopen() {
        // (qty, price): 开 10@100, 减 5@110, 加 5@110, 反手 -15@120, 平 5@115, 重开 5@100, 加 5@100
        let trades = [
            (10., 100.),
            (-5., 110.),
            (5., 110.),
            (-15., 120.),
            (5., 115.),
            (5., 100.),
            (5., 100.),
        ];
        let day = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let time: Vec<Option<NaiveDate>> = (0..trades.len())
            .map(|i| Some(day + chrono::Duration::days(i as i64)))
            .collect();
        let qty: Vec<Option<f64>> = trades.iter().map(|t| Some(t.0)).collect();
        let price: Vec<Option<f64>> = trades.iter().map(|t| Some(t.1)).collect();
        let opt = BondTradePnlOpt {
            bond_info_path: None,
            multiplier: 1.,
            fee: Fee::Zero,
            begin_state: PnlReport::default(),
        };
        let reports =
            calc_bond_trade_pnl(None, &time, &qty, &price, &price, None, None, &opt).unwrap();
        let pos_price: Vec<f64> = reports.iter().map(|r| r.pos_price).collect();
        assert_eq!(pos_price, [100., 100., 105., 120., 0., 100., 100.]);
        for r in &reports {
            assert_eq!(r.avg_price, r.pos_price);
        }
        // 盈亏口径不受影响: 10@100→5@110 实现 50, 10@105→-15@120 实现 150, -5@120→5@115 实现 25
        assert_eq!(reports[6].realized_pnl, 225.);
    }

    /// 在临时目录写一只假券的基础信息, 返回目录 (代码取不存在的, 免得与真实缓存混淆)。
    fn fake_bond_dir(code: &str, json: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("tea_bond_pnl_capital_test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{code}.json")), json).unwrap();
        dir
    }

    fn opt_with(dir: PathBuf, multiplier: f64) -> BondTradePnlOpt {
        BondTradePnlOpt {
            bond_info_path: Some(dir),
            multiplier,
            fee: Fee::Zero,
            begin_state: PnlReport::default(),
        }
    }

    fn d(y: i32, m: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, day)
    }

    #[test]
    fn capital_accrues_on_zero_coupon_bonds() {
        let code = "PNLCAPZC.IB";
        let dir = fake_bond_dir(
            code,
            r#"{"bond_code":"PNLCAPZC.IB","mkt":"IB","abbr":"zc","par_value":100.0,
                "cp_type":"ZeroCoupon","interest_type":"Zero","cp_rate":0.0,"base_rate":null,
                "rate_spread":null,"inst_freq":0,"carry_date":"2025-09-25",
                "maturity_date":"2025-12-25","day_count":"ActAct"}"#,
        );
        // 1000 万面额 (qty 1000 × 乘数 100) 按 99.5 买入, 2% 融资持有 31 天
        let time = vec![d(2025, 10, 1), d(2025, 11, 1)];
        let qty = vec![Some(1000.), Some(0.)];
        let price = vec![Some(99.5), Some(99.6)];
        let rate = vec![Some(0.02), Some(0.02)];
        let reports = calc_bond_trade_pnl(
            Some(code),
            &time,
            &qty,
            &price,
            &price,
            Some(&rate),
            None,
            &opt_with(dir, 100.),
        )
        .unwrap();
        let want = -31. * 0.02 * 1000. * 99.5 * 100. / 365.;
        assert!(
            (reports[1].capital - want).abs() < 1e-6,
            "{}",
            reports[1].capital
        );
    }

    #[test]
    fn capital_base_is_dirty_cost_not_face() {
        let code = "PNLCAPCB.IB";
        let dir = fake_bond_dir(
            code,
            r#"{"bond_code":"PNLCAPCB.IB","mkt":"IB","abbr":"cb","par_value":100.0,
                "cp_type":"CouponBear","interest_type":"Fixed","cp_rate":0.0165,"base_rate":null,
                "rate_spread":null,"inst_freq":1,"carry_date":"2025-06-18",
                "maturity_date":"2035-06-18","day_count":"ActAct"}"#,
        );
        // 净价 105 买入, FR007 1.5% + 20bp, 持有 10 天 (不跨付息日)
        let time = vec![d(2026, 3, 2), d(2026, 3, 12)];
        let qty = vec![Some(1000.), Some(0.)];
        let price = vec![Some(105.), Some(105.)];
        let rate = vec![Some(0.015), Some(0.015)];
        let spread = vec![Some(0.002), None];
        let reports = calc_bond_trade_pnl(
            Some(code),
            &time,
            &qty,
            &price,
            &price,
            Some(&rate),
            Some(&spread),
            &opt_with(dir, 100.),
        )
        .unwrap();
        let dirty = reports[0].pos_price;
        assert!(dirty > 105.5, "成本应是全价, 得到 {dirty}");
        let want = -10. * (0.015 + 0.002) * 1000. * dirty * 100. / 365.;
        assert!(
            (reports[1].capital - want).abs() < 1e-6,
            "{}",
            reports[1].capital
        );
    }

    #[test]
    fn same_day_trades_accrue_once_and_any_row_sets_the_rate() {
        let code = "PNLCAPCB.IB";
        let dir = fake_bond_dir(
            code,
            r#"{"bond_code":"PNLCAPCB.IB","mkt":"IB","abbr":"cb","par_value":100.0,
                "cp_type":"CouponBear","interest_type":"Fixed","cp_rate":0.0165,"base_rate":null,
                "rate_spread":null,"inst_freq":1,"carry_date":"2025-06-18",
                "maturity_date":"2035-06-18","day_count":"ActAct"}"#,
        );
        // 同一天三笔: 买 500 (+20bp, 不带利率) / 买 500 (带 1.5%) / 卖 200; 十天后一行估值
        let time = vec![d(2026, 3, 2), d(2026, 3, 2), d(2026, 3, 2), d(2026, 3, 12)];
        let qty = vec![Some(500.), Some(500.), Some(-200.), Some(0.)];
        let price = vec![Some(105.), Some(105.2), Some(105.1), Some(105.)];
        let rate = vec![None, Some(0.015), None, None];
        let spread = vec![Some(0.002), None, None, None];
        let reports = calc_bond_trade_pnl(
            Some(code),
            &time,
            &qty,
            &price,
            &price,
            Some(&rate),
            Some(&spread),
            &opt_with(dir, 100.),
        )
        .unwrap();
        for r in &reports[..3] {
            assert_eq!(r.capital, 0.);
        }
        let cost = reports[2].pos_price;
        assert_eq!(reports[2].pos, 800.);
        let want = -10. * (0.015 + 0.002) * 800. * cost * 100. / 365.;
        assert!(
            (reports[3].capital - want).abs() < 1e-6,
            "{} vs {want}",
            reports[3].capital
        );
    }

    #[test]
    fn capital_spread_is_signed_and_kept_on_reduce() {
        // 开 1000@+20bp, 加 1000@−40bp → 均值 −10bp; 平 1000 时平仓行带什么加点都不改剩余仓位的加点
        let time: Vec<Option<NaiveDate>> =
            vec![d(2026, 3, 2), d(2026, 3, 3), d(2026, 3, 4), d(2026, 3, 5)];
        let qty = vec![Some(1000.), Some(1000.), Some(-1000.), Some(-500.)];
        let price = vec![Some(100.); 4];
        let spread = vec![Some(0.002), Some(-0.004), Some(0.01), None];
        let opt = BondTradePnlOpt {
            bond_info_path: None,
            multiplier: 1.,
            fee: Fee::Zero,
            begin_state: PnlReport::default(),
        };
        let reports =
            calc_bond_trade_pnl(None, &time, &qty, &price, &price, None, Some(&spread), &opt)
                .unwrap();
        let spreads: Vec<f64> = reports
            .iter()
            .map(|r| (r.avg_capital_spread * 1e4).round())
            .collect();
        assert_eq!(spreads, [20., -10., -10., -10.]);
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct PnlReport {
    pub pos: f64,
    pub avg_price: f64,
    pub pnl: f64,
    pub realized_pnl: f64,
    pub pos_price: f64,
    pub unrealized_pnl: f64,
    pub coupon_paid: f64,
    pub amt: f64,
    pub fee: f64,
    #[serde(default)]
    pub avg_capital_spread: f64, // 平均资金加点
    #[serde(default)]
    pub capital: f64, // 累计资金
}

#[derive(Deserialize)]
pub struct BondTradePnlOpt {
    pub bond_info_path: Option<PathBuf>,
    pub multiplier: f64,
    pub fee: Fee,
    pub begin_state: PnlReport,
}

pub fn calc_bond_trade_pnl<T, V, VT>(
    symbol: Option<&str>,
    settle_time_vec: &VT,
    qty_vec: &V,
    clean_price_vec: &V,
    clean_close_vec: &V,
    capital_rate_vec: Option<&V>,
    capital_spread_vec: Option<&V>,
    opt: &BondTradePnlOpt,
) -> Result<Vec<PnlReport>>
where
    T: IsNone,
    T::Inner: Number,
    V: Vec1View<T>,
    VT: Vec1View<Option<NaiveDate>>,
{
    if qty_vec.is_empty() {
        return Ok(Vec::empty());
    }
    let multiplier = opt.multiplier;
    let mut state = opt.begin_state;
    let mut last_settle_time = None;
    let mut last_capital_rate = 0.;
    let mut last_cp_date = Default::default();
    let mut accrued_interest = 0.;
    let mut next_day_coupon: f64 = 0.;
    let mut duration_days = 0.; // 用于统计不同天的交易记录中间的天数间隔
    let symbol = if let Some(bond) = symbol {
        if bond.is_empty() {
            None
        } else {
            CachedBond::new(bond, opt.bond_info_path.as_deref()).ok()
        }
    } else {
        None
    };
    let coupon_paid = symbol.as_ref().map(|bond| bond.get_coupon()).unwrap_or(0.);
    let capital_rate_vec = match capital_rate_vec {
        Some(v) => Either::Left(
            v.titer()
                .map(|v| if v.is_none() { 0. } else { v.unwrap().f64() }),
        ),
        None => Either::Right(std::iter::repeat(0.)),
    };
    let capital_spread_vec = match capital_spread_vec {
        Some(v) => Either::Left(
            v.titer()
                .map(|v| if v.is_none() { 0. } else { v.unwrap().f64() }),
        ),
        None => Either::Right(std::iter::repeat(0.)),
    };
    izip!(
        settle_time_vec.titer(),
        qty_vec.titer(),
        clean_price_vec.titer(),
        clean_close_vec.titer(),
        capital_rate_vec,
        capital_spread_vec,
    )
    .map(
        |(settle_time, qty, clean_price, clean_close, capital_rate, capital_spread)| {
            let qty = if qty.is_none() {
                0.
            } else {
                qty.unwrap().f64()
            };

            let (trade_price, close): (Option<f64>, Option<f64>) = if let Some(bond) = &symbol {
                let zero_coupon = bond.is_zero_coupon();
                let settle_time = match settle_time {
                    Some(t) => Some(t),
                    // 零息券过去整段跳过, 没有结算日也能算价差; 只是这样就记不了资金成本
                    None if zero_coupon => None,
                    None => {
                        return Err(anyhow!(
                            "Settle time should not be none when calc bond trade pnl"
                        ));
                    }
                };
                // === 新的一天重新计算持仓的相关信息
                if let Some(settle_time) = settle_time
                    && last_settle_time != Some(settle_time)
                {
                    // 上一段持仓的资金成本. 计息基数是持仓的全价成本 (pos_price, 即融入的金额),
                    // 不是面额; 零息/贴现券同样计 —— 过去这段写在付息债分支里, 零息券一分不扣.
                    duration_days = last_settle_time
                        .map(|t| settle_time.signed_duration_since(t).num_days() as f64)
                        .unwrap_or(0.);
                    state.capital -= duration_days
                        * (last_capital_rate * state.pos
                            + state.avg_capital_spread * state.pos.abs())
                        * state.pos_price
                        * multiplier
                        / 365.;
                    if !zero_coupon {
                        if let Some(last_settle_time) = last_settle_time {
                            let coupon_count =
                                coupon_payment_count_between(bond, last_settle_time, settle_time)?;
                            state.coupon_paid +=
                                coupon_count as f64 * coupon_paid * multiplier * state.pos;
                        }
                        if next_day_coupon != 0. {
                            state.coupon_paid += next_day_coupon;
                            next_day_coupon = 0.;
                        }
                        // 新的一天重新计算相关信息
                        let cp_dates = bond.get_nearest_cp_date(settle_time)?;
                        accrued_interest =
                            bond.calc_accrued_interest(settle_time, Some(cp_dates))?;
                        last_cp_date = bond.mkt.find_workday(cp_dates.0, 0);
                        // 当天初始仓位会产生的票息
                        if settle_time == last_cp_date {
                            // 调节应计利息
                            accrued_interest = coupon_paid;
                            next_day_coupon += coupon_paid * multiplier * state.pos;
                        }
                    }
                    last_settle_time = Some(settle_time);
                }
                // 利率在上一段结转之后才换, 当天任一行给的都算 (过去只认当天第一行)
                if capital_rate != 0. {
                    last_capital_rate = capital_rate;
                }
                // === 交易当天付息调整
                if !zero_coupon && (settle_time == Some(last_cp_date)) & (qty != 0.) {
                    next_day_coupon += coupon_paid * multiplier * qty;
                }
                (
                    clean_price.map(|v| v.f64() + accrued_interest),
                    clean_close.map(|v| v.f64() + accrued_interest),
                )
            } else {
                (clean_price.map(|v| v.f64()), clean_close.map(|v| v.f64()))
            };
            if qty != 0. {
                let trade_price = trade_price
                    .ok_or_else(|| anyhow!("Trade price should not be none"))?
                    .f64();
                let prev_pos = state.pos;
                let trade_amt = qty * trade_price * multiplier; // with sign
                state.pos += qty;
                state.amt += trade_amt;
                state.fee += opt.fee.amount(qty, trade_amt, 1); // Fee model will take into account the sign of the trade amount and quantity.
                if prev_pos.abs() > EPS {
                    if qty.signum() != prev_pos.signum() {
                        // 减仓
                        let qty_chg = qty.abs().min(prev_pos.abs()) * prev_pos.signum();
                        state.realized_pnl +=
                            (trade_price - state.pos_price) * multiplier * qty_chg;
                        if qty.abs() > prev_pos.abs() {
                            // 反向开仓
                            state.pos_price = trade_price;
                            state.avg_capital_spread = capital_spread;
                        }
                        // 减仓: 剩下那部分的加点不变, 与 pos_price 同口径 (按均值平, 不按批次).
                        // 过去拿平仓行的加点去倒扣均值, 平仓行带的加点与均值不同就会把剩余仓位的
                        // 加点改坏, 甚至扣成负数.
                    } else {
                        // 加仓
                        state.pos_price = (state.pos_price * prev_pos.abs()
                            + qty.abs() * trade_price)
                            / state.pos.abs();
                        if capital_spread != 0. {
                            // 加点带符号 (FR007 − 10bp 是负加点); 过去取了绝对值
                            state.avg_capital_spread = (state.avg_capital_spread * prev_pos.abs()
                                + capital_spread * qty.abs())
                                / state.pos.abs();
                        }
                    }
                    if state.pos.abs() <= EPS {
                        state.pos_price = 0.;
                        state.avg_capital_spread = 0.;
                    }
                } else {
                    // 之前仓位是0
                    state.pos_price = trade_price;
                    state.avg_capital_spread = capital_spread;
                }
            }
            // avg_price 保留为 pos_price 的别名(当前持仓的开仓均价), 只为输出结构不变.
            // 旧实现只在加仓时写成 amt / (pos * multiplier), 减仓/反手不更新, 且 amt
            // 从序列起点累计, 平仓后重开再加仓会把之前所有回合的盈亏摊进来.
            state.avg_price = state.pos_price;
            if let Some(close) = close {
                let close = close.f64();
                state.pnl =
                    state.pos * close * multiplier + state.coupon_paid - state.amt - state.fee
                        + state.capital;
                state.unrealized_pnl = state.pos * (close - state.pos_price) * multiplier;
            }
            // println!("PNL Report: {:?}", state);
            Ok(state)
        },
    )
    .collect()
}
