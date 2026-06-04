use std::fmt::Debug;

use itertools::{Either, izip};
use serde::Deserialize;
use tevec::prelude::{EPS, IsNone, Number, Vec1View};

#[derive(Default, Debug)]
pub struct Trade<T> {
    pub time: T,
    pub price: f64,
    pub qty: f64,
}

impl<T: PartialEq + Debug> std::ops::Add<Trade<T>> for Trade<T> {
    type Output = Trade<T>;
    #[inline]
    fn add(self, rhs: Trade<T>) -> Trade<T> {
        if (self.time == rhs.time) && (self.price == rhs.price) {
            Trade {
                time: self.time,
                price: self.price,
                qty: self.qty + rhs.qty,
            }
        } else if self.time == rhs.time {
            let amt = self.qty * self.price + rhs.qty * rhs.price;
            let price = if self.qty + rhs.qty != 0. {
                amt / (self.qty + rhs.qty)
            } else {
                amt
            };
            Trade {
                time: self.time,
                price,
                qty: self.qty + rhs.qty,
            }
        } else {
            panic!("trade time or price is not equal, {self:?} != {rhs:?}");
        }
    }
}

impl<T: PartialEq + Debug> std::ops::Add<Option<Trade<T>>> for Trade<T> {
    type Output = Trade<T>;
    #[inline]
    fn add(self, rhs: Option<Trade<T>>) -> Trade<T> {
        if let Some(rt) = rhs { self + rt } else { self }
    }
}

#[derive(Deserialize)]
pub struct TradeFromPosOpt {
    pub cash: Option<f64>,
    pub multiplier: f64,
    pub qty_tick: f64,
    #[serde(default)]
    pub qty_round_mode: QtyRoundMode,
    pub stop_on_finish: bool,
    #[serde(default)]
    pub finish_price: Option<f64>,
    #[serde(default)]
    pub min_adjust_amt: Option<f64>,
    #[serde(default)]
    pub keep_shape: Option<bool>,
}

const INIT_TRADE_COUNT: usize = 512;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QtyRoundMode {
    #[default]
    Floor,
    Round,
}

fn quantize(q: f64, tick: f64, mode: QtyRoundMode) -> f64 {
    if tick <= 0.0 {
        return q;
    }
    match mode {
        QtyRoundMode::Floor => {
            if q >= 0.0 {
                (q / tick).floor() * tick
            } else {
                (q / tick).ceil() * tick
            }
        }
        QtyRoundMode::Round => (q / tick).round() * tick,
    }
}

fn to_opt_f64<T: IsNone>(v: T) -> Option<f64>
where
    T::Inner: Number,
{
    if v.is_none() {
        None
    } else {
        Some(v.unwrap().f64())
    }
}

fn broadcast<'a, T, V>(
    vec: &'a V,
) -> Either<std::iter::Repeat<Option<f64>>, impl Iterator<Item = Option<f64>> + 'a>
where
    T: IsNone + 'a,
    T::Inner: Number,
    V: Vec1View<T>,
{
    if vec.len() == 1 {
        Either::Left(std::iter::repeat(to_opt_f64(vec.titer().next().unwrap())))
    } else {
        Either::Right(vec.titer().map(to_opt_f64))
    }
}

#[allow(clippy::collapsible_if)]
pub fn trading_from_pos<DT, T, VT, V>(
    time_vec: &VT,
    pos_vec: &V,
    open_vec: &V,
    opt: &TradeFromPosOpt,
) -> Vec<Option<Trade<DT>>>
where
    DT: Clone + PartialEq + Debug,
    T: IsNone,
    T::Inner: Number,
    VT: Vec1View<DT>,
    V: Vec1View<T>,
{
    let mut open_qty: f64 = 0.;
    let cash = opt.cash.unwrap();
    let min_adjust_amt = opt.min_adjust_amt.unwrap_or(0.);
    let mut trades = Vec::with_capacity(INIT_TRADE_COUNT);
    let keep_shape = opt.keep_shape.unwrap_or(false);

    // 记录最后一个可用 (time, price)，用于 stop_on_finish
    let mut last_tp: Option<(DT, f64)> = None;

    // pos_vec / open_vec 长度为 1 时广播为常量，支持标量字面量传入
    let pos_iter = broadcast(pos_vec);
    let open_iter = broadcast(open_vec);

    izip!(time_vec.titer(), pos_iter, open_iter).for_each(|(time, pos, open)| {
        if let (Some(pos), Some(price)) = (pos, open) {
            // 记录最新可用的时间与价格
            last_tp = Some((time.clone(), price));

            // 先用目标仓位计算目标手数，再用真实持仓手数补齐差额。
            let target_qty = if pos.abs() > EPS {
                let raw_qty = pos * cash / (price * opt.multiplier);
                quantize(raw_qty, opt.qty_tick, opt.qty_round_mode)
            } else {
                0.0
            };
            let qty = target_qty - open_qty;

            if qty.abs() > EPS {
                let adjust_amt = qty.abs() * price * opt.multiplier;

                if adjust_amt > min_adjust_amt {
                    trades.push(Some(Trade {
                        time: time.clone(),
                        price,
                        qty,
                    }));
                    open_qty += qty;
                } else if keep_shape {
                    trades.push(None);
                }
            } else if keep_shape {
                trades.push(None);
            }
        } else if keep_shape {
            trades.push(None)
        }
    });

    // 收尾是否强制平仓
    if opt.stop_on_finish && (open_qty != 0.0) {
        if let Some((t, p)) = last_tp {
            let p = opt.finish_price.unwrap_or(p);
            let trade = Trade {
                time: t,
                price: p,
                qty: -open_qty,
            };
            if keep_shape {
                let last_trade = trades.pop().flatten();
                trades.push(Some(trade + last_trade));
            } else {
                trades.push(Some(trade))
            }
        }
    }
    trades
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(qty_round_mode: QtyRoundMode) -> TradeFromPosOpt {
        TradeFromPosOpt {
            cash: Some(100_000.0),
            multiplier: 1.0,
            qty_tick: 1000.0,
            qty_round_mode,
            stop_on_finish: false,
            finish_price: None,
            min_adjust_amt: None,
            keep_shape: Some(true),
        }
    }

    fn qtys(trades: Vec<Option<Trade<i32>>>) -> Vec<Option<f64>> {
        trades
            .into_iter()
            .map(|trade| trade.map(|trade| trade.qty))
            .collect()
    }

    #[test]
    fn uses_target_qty_instead_of_pos_delta() {
        let time = vec![1, 2, 3, 4, 5];
        let pos = vec![0.19, 0.38, 0.57, 0.76, 0.95];
        let open = vec![10.0; 5];

        let trades = trading_from_pos(&time, &pos, &open, &opt(QtyRoundMode::Floor));

        assert_eq!(
            qtys(trades),
            vec![
                Some(1000.0),
                Some(2000.0),
                Some(2000.0),
                Some(2000.0),
                Some(2000.0),
            ]
        );
    }

    #[test]
    fn floor_mode_truncates_signed_qty_toward_zero() {
        let time = vec![1, 2];
        let pos = vec![0.19, -0.19];
        let open = vec![10.0, 10.0];

        let trades = trading_from_pos(&time, &pos, &open, &opt(QtyRoundMode::Floor));

        assert_eq!(qtys(trades), vec![Some(1000.0), Some(-2000.0)]);
    }

    #[test]
    fn round_mode_rounds_target_qty_to_nearest_tick() {
        let time = vec![1, 2];
        let pos = vec![0.16, -0.16];
        let open = vec![10.0, 10.0];

        let trades = trading_from_pos(&time, &pos, &open, &opt(QtyRoundMode::Round));

        assert_eq!(qtys(trades), vec![Some(2000.0), Some(-4000.0)]);
    }

    #[test]
    fn skipped_adjustment_does_not_update_current_qty() {
        let time = vec![1, 2];
        let pos = vec![0.11, 0.21];
        let open = vec![10.0, 10.0];
        let mut opt = opt(QtyRoundMode::Floor);
        opt.min_adjust_amt = Some(15_000.0);

        let trades = trading_from_pos(&time, &pos, &open, &opt);

        assert_eq!(qtys(trades), vec![None, Some(2000.0)]);
    }
}
