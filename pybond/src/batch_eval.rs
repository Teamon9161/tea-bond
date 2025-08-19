use chrono::{Datelike, Days, NaiveDate};
use pyo3_polars::derive::polars_expr;
use pyo3_polars::export::polars_core::utils::CustomIterTools;
use serde::Deserialize;
use tea_bond::export::calendar::Calendar;
use tea_bond::{BondYtm, CachedBond, Future, Market, TfEvaluator};
use tevec::export::arrow as polars_arrow;
use tevec::export::polars::prelude::*;

#[derive(Deserialize)]
struct EvaluatorBatchParams {
    pub reinvest_rate: Option<f64>,
}

macro_rules! auto_cast {
    // for one expression
    ($arm: ident ($se: expr)) => {
        if let DataType::$arm = $se.dtype() {
            $se
        } else {
            &$se.cast(&DataType::$arm)?
        }
    };
    // for multiple expressions
    ($arm: ident ($($se: expr),*)) => {
        ($(
            if let DataType::$arm = $se.dtype() {
                $se
            } else {
                &$se.cast(&DataType::$arm)?
            }
        ),*)
    };
}

pub const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
pub const EPOCH_DAYS_FROM_CE: i32 = 719163;

fn batch_eval_impl<F1, F2, O>(
    future: &StringChunked,
    bond: &StringChunked,
    date: &DateChunked,
    future_price: &Float64Chunked,
    bond_ytm: &Float64Chunked,
    capital_rate: &Float64Chunked,
    reinvest_rate: Option<f64>,
    evaluator_func: F1,
    return_func: F2,
    null_future_return_null: bool,
    null_bond_return_null: bool,
) -> Vec<Option<O>>
where
    F1: Fn(TfEvaluator) -> TfEvaluator,
    F2: Fn(&TfEvaluator) -> Option<O>, // O: PolarsDataType,
{
    let reinvest_rate = Some(reinvest_rate.unwrap_or(0.0));
    let len_vec = [
        future_price.len(),
        bond_ytm.len(),
        bond.len(),
        future.len(),
        date.len(),
    ];
    let len = *len_vec.iter().max().unwrap();
    if *len_vec.iter().min().unwrap() == 0 {
        return Default::default();
    }
    // get iterators
    let mut future_iter = future.iter();
    let mut future_price_iter = future_price.iter();
    let mut bond_iter = bond.iter();
    let mut bond_ytm_iter = bond_ytm.iter();
    let mut capital_rate_iter = capital_rate.iter();
    let mut date_iter = date.physical().iter();

    let mut result = Vec::with_capacity(len);
    let mut future: Arc<Future> = Future::new(future_iter.next().unwrap().unwrap_or("")).into();
    let mut future_price = future_price_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut bond = CachedBond::new(bond_iter.next().unwrap().unwrap_or(""), None).unwrap();
    let mut bond_ytm = bond_ytm_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut date_physical = date_iter.next().unwrap().unwrap_or(0);
    let mut date = EPOCH
        .checked_add_days(Days::new(date_physical as u64))
        .unwrap();
    let mut capital_rate = capital_rate_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut evaluator = TfEvaluator {
        date,
        future: (future.clone(), future_price).into(),
        bond: BondYtm::new(bond.clone(), bond_ytm),
        capital_rate,
        reinvest_rate,
        ..Default::default()
    };
    if (null_future_return_null && evaluator.future.code.is_empty())
        || (null_bond_return_null && evaluator.bond.code().is_empty())
    {
        result.push(None);
    } else {
        evaluator = evaluator_func(evaluator);
        result.push(return_func(&evaluator));
    }
    for _ in 1..len {
        if let Some(fp) = future_price_iter.next() {
            future_price = fp.unwrap_or(f64::NAN);
        };
        if let Some(by) = bond_ytm_iter.next() {
            bond_ytm = by.unwrap_or(f64::NAN);
        };
        if let Some(cy) = capital_rate_iter.next() {
            capital_rate = cy.unwrap_or(f64::NAN);
        };
        if let Some(dt) = date_iter.next() {
            let dt = dt.unwrap_or(0);
            if dt != date_physical {
                date_physical = dt;
                date = EPOCH.checked_add_days(Days::new(dt as u64)).unwrap()
            }
        };
        if let Some(f) = future_iter.next() {
            if let Some(f) = f {
                if future.code != f {
                    future = Future::new(f).into()
                }
            } else {
                // TODO(Teamon): 期货如果为空，可能影响结果正确性，最好有进一步的处理
                if null_future_return_null {
                    result.push(None);
                    bond_iter.next(); // 由于提前continue, 必须手动迭代bond以匹配对应行
                    continue;
                }
                future = Default::default();
            }
        };
        if let Some(b) = bond_iter.next() {
            if let Some(b) = b {
                if b != bond.code() && bond.bond_code != b {
                    bond = CachedBond::new(b, None).unwrap();
                }
            } else {
                if null_bond_return_null {
                    result.push(None);
                    continue;
                }
                bond = Default::default();
            }
        };

        evaluator = evaluator.update_with_new_info(
            date,
            (future.clone(), future_price),
            (bond.clone(), bond_ytm),
            capital_rate,
            reinvest_rate,
        );
        if (null_future_return_null && evaluator.future.code.is_empty())
            || (null_bond_return_null && evaluator.bond.code().is_empty())
        {
            result.push(None);
            continue;
        }
        // dbg!("{} {} {}", i, date, &bond.bond_code);
        evaluator = evaluator_func(evaluator);
        result.push(return_func(&evaluator));
    }
    result
}

fn batch_eval<F1, F2, O>(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
    evaluator_func: F1,
    return_func: F2,
    null_future_return_null: bool,
    null_bond_return_null: bool,
) -> PolarsResult<Vec<Option<O>>>
where
    F1: Fn(TfEvaluator) -> TfEvaluator,
    F2: Fn(&TfEvaluator) -> Option<O>,
{
    let (future, bond, date, future_price, bond_ytm, capital_rate) = (
        &inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4], &inputs[5],
    );
    let (future_price, bond_ytm, capital_rate) =
        auto_cast!(Float64(future_price, bond_ytm, capital_rate));
    let date = auto_cast!(Date(date));
    Ok(batch_eval_impl(
        future.str()?,
        bond.str()?,
        date.date()?,
        future_price.f64()?,
        bond_ytm.f64()?,
        capital_rate.f64()?,
        kwargs.reinvest_rate,
        evaluator_func,
        return_func,
        null_future_return_null,
        null_bond_return_null,
    ))
}

#[polars_expr(output_type=Float64)]
fn evaluators_net_basis_spread(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_net_basis_spread().unwrap(),
        |e: &TfEvaluator| e.net_basis_spread,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_accrued_interest(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_accrued_interest().unwrap(),
        |e: &TfEvaluator| e.accrued_interest,
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_deliver_accrued_interest(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_deliver_accrued_interest().unwrap(),
        |e: &TfEvaluator| e.deliver_accrued_interest,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_cf(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_cf().unwrap(),
        |e: &TfEvaluator| e.cf,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_dirty_price(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_dirty_price().unwrap(),
        |e: &TfEvaluator| e.dirty_price,
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_clean_price(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_clean_price().unwrap(),
        |e: &TfEvaluator| e.clean_price,
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_future_dirty_price(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_future_dirty_price().unwrap(),
        |e: &TfEvaluator| e.future_dirty_price,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_deliver_cost(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_deliver_cost().unwrap(),
        |e: &TfEvaluator| e.deliver_cost,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_basis_spread(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_basis_spread().unwrap(),
        |e: &TfEvaluator| e.basis_spread,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_f_b_spread(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_f_b_spread().unwrap(),
        |e: &TfEvaluator| e.f_b_spread,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_carry(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_carry().unwrap(),
        |e: &TfEvaluator| e.carry,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_duration(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_duration().unwrap(),
        |e: &TfEvaluator| e.duration,
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_irr(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_irr().unwrap(),
        |e: &TfEvaluator| e.irr,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_future_ytm(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_future_ytm().unwrap(),
        |e: &TfEvaluator| e.future_ytm,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_remain_cp_to_deliver(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_remain_cp_to_deliver().unwrap(),
        |e: &TfEvaluator| e.remain_cp_to_deliver,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn evaluators_remain_cp_to_deliver_wm(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_remain_cp_to_deliver().unwrap(),
        |e: &TfEvaluator| e.remain_cp_to_deliver_wm,
        true,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Int32)]
fn evaluators_remain_cp_num(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Int32Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_remain_cp_num().unwrap(),
        |e: &TfEvaluator| e.remain_cp_num,
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Date)]
fn evaluators_deliver_date(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Int32Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_deliver_date().unwrap(),
        |e: &TfEvaluator| {
            e.deliver_date
                .map(|d| d.num_days_from_ce() - EPOCH_DAYS_FROM_CE)
        },
        true,
        false,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_date().into_series())
}

#[polars_expr(output_type=Date)]
fn evaluators_last_trading_date(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Int32Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e,
        |e: &TfEvaluator| {
            e.future
                .last_trading_date()
                .ok()
                .map(|d| d.num_days_from_ce() - EPOCH_DAYS_FROM_CE)
        },
        true,
        false,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_date().into_series())
}

#[derive(Deserialize)]
struct FindWorkdayKwargs {
    market: Market,
    offset: i32,
}

#[polars_expr(output_type=Date)]
fn calendar_find_workday(inputs: &[Series], kwargs: FindWorkdayKwargs) -> PolarsResult<Series> {
    use tea_bond::export::calendar::china;
    let date_col = auto_cast!(Date(&inputs[0]));
    let date_series = date_col.date()?.physical();
    let res: Int32Chunked = match kwargs.market {
        Market::IB => date_series
            .iter()
            .map(|value| {
                value.map(|v| {
                    let dt = EPOCH.checked_add_days(Days::new(v as u64)).unwrap();
                    china::IB.find_workday(dt, kwargs.offset).num_days_from_ce()
                        - EPOCH_DAYS_FROM_CE
                })
            })
            .collect_trusted(),
        Market::SSE | Market::SH | Market::SZ | Market::SZE => date_series
            .iter()
            .map(|value| {
                value.map(|v| {
                    let dt = EPOCH.checked_add_days(Days::new(v as u64)).unwrap();
                    china::SSE
                        .find_workday(dt, kwargs.offset)
                        .num_days_from_ce()
                        - EPOCH_DAYS_FROM_CE
                })
            })
            .collect_trusted(),
    };
    Ok(res.into_date().into_series())
}

#[derive(Deserialize)]
struct IsBusinessDayKwargs {
    market: Market,
}

#[polars_expr(output_type=Boolean)]
fn calendar_is_business_day(
    inputs: &[Series],
    kwargs: IsBusinessDayKwargs,
) -> PolarsResult<Series> {
    use tea_bond::export::calendar::china;
    let date_col = auto_cast!(Date(&inputs[0]));
    let date_series = date_col.date()?.physical();
    let res: BooleanChunked = match kwargs.market {
        Market::IB => date_series
            .iter()
            .map(|value| {
                value.map(|v| {
                    let dt = EPOCH.checked_add_days(Days::new(v as u64)).unwrap();
                    china::IB.is_business_day(dt)
                })
            })
            .collect_trusted(),
        Market::SSE | Market::SH | Market::SZ | Market::SZE => date_series
            .iter()
            .map(|value| {
                value.map(|v| {
                    let dt = EPOCH.checked_add_days(Days::new(v as u64)).unwrap();
                    china::SSE.is_business_day(dt)
                })
            })
            .collect_trusted(),
    };
    Ok(res.into_series())
}
