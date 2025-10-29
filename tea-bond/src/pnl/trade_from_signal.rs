use itertools::izip;
use serde::Deserialize;
use tevec::prelude::{EPS, IsNone, Number, Vec1View};
// use super::EPOCH;

pub struct Trade<T> {
    pub time: T,
    pub price: f64,
    pub qty: f64,
}

#[derive(Deserialize)]
pub struct TradeFromPosOpt {
    pub cash: Option<f64>,
    pub multiplier: f64,
    pub qty_tick: f64,
    pub stop_on_finish: bool,
    pub finish_price: Option<f64>,
}

const INIT_TRADE_COUNT: usize = 512;

fn quantize_inside(q: f64, tick: f64) -> f64 {
    if tick <= 0.0 {
        return q;
    }
    if q >= 0.0 {
        (q / tick).floor() * tick // 买单向下取整
    } else {
        (q / tick).ceil() * tick // 卖单向上取整（数值更接近 0）
    }
}

pub fn trading_from_pos<DT, T, VT, V>(
    time_vec: &VT,
    pos_vec: &V,
    open_vec: &V,
    opt: &TradeFromPosOpt,
) -> Vec<Trade<DT>>
where
    DT: Clone,
    T: IsNone,
    T::Inner: Number,
    VT: Vec1View<DT>,
    V: Vec1View<T>,
{
    let mut last_pos = 0.;
    let mut open_price: f64 = 0.;
    let mut open_qty: f64 = 0.;
    let cash = opt.cash.unwrap();
    let mut trades = Vec::with_capacity(INIT_TRADE_COUNT);

    // 记录最后一个可用 (time, price)，用于 stop_on_finish
    let mut last_tp: Option<(DT, f64)> = None;

    izip!(time_vec.titer(), pos_vec.titer(), open_vec.titer()).for_each(|(time, pos, open)| {
        if pos.not_none() && open.not_none() {
            let pos = pos.unwrap().f64();
            let price = open.unwrap().f64();
            // 记录最新可用的时间与价格
            last_tp = Some((time.clone(), price));

            let dpos = pos - last_pos;
            if dpos.abs() > EPS {
                // 目标名义 -> 成交量（正买负卖）
                let qty = if pos.abs() > EPS {
                    let p = if open_price > 0. { open_price } else { price };
                    let raw_qty = dpos * cash / (p * opt.multiplier);
                    // 量化到最小变动单位（朝 0 截断，避免超买/超卖）
                    quantize_inside(raw_qty, opt.qty_tick)
                } else {
                    -open_qty
                };
                if open_qty == 0. {
                    // 开仓情况
                    open_price = price;
                } else if dpos.signum() == open_qty.signum() {
                    open_price = (open_price * open_qty + qty * price) / (qty + open_qty);
                } else if open_qty.abs() > qty.abs() {
                    // 反向加仓, 价格为新的开仓价格
                    open_price = price;
                };
                // 减仓情况的价格不改变

                // 若量化后仍非 0，则下单
                if qty.abs() > 0.0 {
                    trades.push(Trade {
                        time: time.clone(),
                        price,
                        qty,
                    });
                    open_qty += qty;
                }
            }

            // 维持 last_pos 为目标（策略）层面的持仓比例/规模
            last_pos = pos;
        }
    });

    // 收尾是否强制平仓
    if opt.stop_on_finish && (open_qty != 0.0) {
        if let Some((t, p)) = last_tp {
            let p = if let Some(p) = opt.finish_price { p } else { p };
            trades.push(Trade {
                time: t,
                price: p,
                qty: -open_qty,
            });
        }
    }

    trades
}
