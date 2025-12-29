use anyhow::Result;
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
) -> PolarsResult<Vec<Option<O>>>
where
    F1: Fn(TfEvaluator) -> Result<TfEvaluator>,
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
        return Ok(Default::default());
    }
    // get iterators
    let mut future_iter = future.iter();
    let mut future_price_iter = future_price.iter();
    let mut bond_iter = bond.iter();
    let mut bond_ytm_iter = bond_ytm.iter();
    let mut capital_rate_iter = capital_rate.iter();
    let mut date_iter = date.as_date_iter();

    let mut result = Vec::with_capacity(len);
    let mut future: Arc<Future> = Future::new(future_iter.next().unwrap().unwrap_or("")).into();
    let mut future_price = future_price_iter.next().unwrap().unwrap_or(f64::NAN);
    // allow unknown bond
    let mut bond =
        CachedBond::new(bond_iter.next().unwrap().unwrap_or(""), None).unwrap_or_default();
    let mut bond_ytm = bond_ytm_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut date = date_iter.next().unwrap().unwrap_or_default();
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
        evaluator = evaluator_func(evaluator).map_err(|e| {
            PolarsError::ComputeError(format!("Evaluator function error: {}", e).into())
        })?;
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
            date = dt.unwrap_or_default();
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
                    bond = CachedBond::new(b, None).unwrap_or_default();
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
        evaluator = evaluator_func(evaluator)
            .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
        result.push(return_func(&evaluator));
    }
    Ok(result)
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
    F1: Fn(TfEvaluator) -> Result<TfEvaluator>,
    F2: Fn(&TfEvaluator) -> Option<O>,
{
    let (future, bond, date, future_price, bond_ytm, capital_rate) = (
        &inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4], &inputs[5],
    );
    let (future_price, bond_ytm, capital_rate) =
        auto_cast!(Float64(future_price, bond_ytm, capital_rate));
    let date = auto_cast!(Date(date));
    let bond = auto_cast!(String(bond));
    batch_eval_impl(
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
    )
}

#[polars_expr(output_type=Float64)]
fn evaluators_net_basis_spread(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        |e: TfEvaluator| e.with_net_basis_spread(),
        |e: &TfEvaluator| e.net_basis_spread.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_accrued_interest(),
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
        |e: TfEvaluator| e.with_deliver_accrued_interest(),
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
        |e: TfEvaluator| e.with_cf(),
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
        |e: TfEvaluator| e.with_dirty_price(),
        |e: &TfEvaluator| e.dirty_price.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_clean_price(),
        |e: &TfEvaluator| e.clean_price.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_future_dirty_price(),
        |e: &TfEvaluator| e.future_dirty_price.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_deliver_cost(),
        |e: &TfEvaluator| e.deliver_cost.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_basis_spread(),
        |e: &TfEvaluator| e.basis_spread.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_f_b_spread(),
        |e: &TfEvaluator| e.f_b_spread.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_carry(),
        |e: &TfEvaluator| e.carry.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_duration(),
        |e: &TfEvaluator| e.duration.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_irr(),
        |e: &TfEvaluator| e.irr.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_future_ytm(),
        |e: &TfEvaluator| e.future_ytm.filter(|v| !v.is_nan()),
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
        |e: TfEvaluator| e.with_remain_cp_to_deliver(),
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
        |e: TfEvaluator| e.with_remain_cp_to_deliver(),
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
        |e: TfEvaluator| Ok(e.with_remain_cp_num()),
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
    let result = DateChunked::from_naive_date_options(
        "".into(),
        batch_eval(
            inputs,
            kwargs,
            |e: TfEvaluator| e.with_deliver_date(),
            |e: &TfEvaluator| e.deliver_date,
            true,
            false,
        )?
        .into_iter(),
    );
    Ok(result.into_series())
}

#[polars_expr(output_type=Date)]
fn evaluators_last_trading_date(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let result = DateChunked::from_naive_date_options(
        "".into(),
        batch_eval(
            inputs,
            kwargs,
            Ok,
            |e: &TfEvaluator| e.future.last_trading_date().ok(),
            true,
            false,
        )?
        .into_iter(),
    );
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn bonds_remain_year(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result: Float64Chunked = batch_eval(
        inputs,
        kwargs,
        Ok,
        |e: &TfEvaluator| Some(e.bond.remain_year(e.date)),
        false,
        true,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}

#[polars_expr(output_type=Date)]
fn bonds_carry_date(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result = DateChunked::from_naive_date_options(
        "".into(),
        batch_eval(
            inputs,
            kwargs,
            Ok,
            |e: &TfEvaluator| Some(e.bond.carry_date),
            false,
            true,
        )?
        .into_iter(),
    );
    Ok(result.into_series())
}

#[polars_expr(output_type=Date)]
fn bonds_maturity_date(inputs: &[Series], kwargs: EvaluatorBatchParams) -> PolarsResult<Series> {
    let result = DateChunked::from_naive_date_options(
        "".into(),
        batch_eval(
            inputs,
            kwargs,
            Ok,
            |e: &TfEvaluator| Some(e.bond.maturity_date),
            false,
            true,
        )?
        .into_iter(),
    );
    Ok(result.into_series())
}

#[polars_expr(output_type=Float64)]
fn bonds_calc_ytm_with_price(inputs: &[Series]) -> PolarsResult<Series> {
    let dirty_price_se = auto_cast!(Float64(&inputs[2]));
    let bond_se = auto_cast!(String(&inputs[0]));
    let date_se = auto_cast!(Date(&inputs[1]));
    let len = dirty_price_se.len();
    if len == 0 {
        return Ok(Default::default());
    }
    let bond = bond_se.str()?;
    let dirty_price = dirty_price_se.f64()?;
    let mut bond_iter = bond.iter();
    let mut date_iter = date_se.date()?.as_date_iter();
    let mut bond =
        CachedBond::new(bond_iter.next().unwrap().unwrap_or(""), None).unwrap_or_default();
    let mut dirty_price_iter = dirty_price.iter();
    let mut date = date_iter.next().unwrap().unwrap_or_default();

    let mut dirty_price = dirty_price_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut result = Vec::with_capacity(len);
    if bond.bond_code().is_empty() {
        result.push(None)
    } else {
        result.push(
            bond.calc_ytm_with_price(dirty_price, date, None, None)
                .ok()
                .filter(|v| !v.is_nan()),
        )
    }
    for _ in 1..len {
        if let Some(dp) = dirty_price_iter.next() {
            dirty_price = dp.unwrap_or(f64::NAN);
        };
        if let Some(dt) = date_iter.next() {
            date = dt.unwrap_or_default()
        };
        if let Some(b) = bond_iter.next() {
            if let Some(b) = b {
                if b != bond.code() && bond.bond_code != b {
                    bond = CachedBond::new(b, None).unwrap_or_default();
                }
            } else {
                result.push(None);
                continue;
            }
        };
        if bond.bond_code().is_empty() {
            result.push(None);
        } else {
            result.push(
                bond.calc_ytm_with_price(dirty_price, date, None, None)
                    .ok()
                    .filter(|v| !v.is_nan()),
            )
        }
    }
    let result: Float64Chunked = result.into_iter().collect_trusted();
    Ok(result.into_series())
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
    let date_series = date_col.date()?;
    let res = match kwargs.market {
        Market::IB => DateChunked::from_naive_date_options(
            "".into(),
            date_series
                .as_date_iter()
                .map(|value| value.map(|date| china::IB.find_workday(date, kwargs.offset))),
        ),
        Market::SSE | Market::SH | Market::SZ | Market::SZE => {
            DateChunked::from_naive_date_options(
                "".into(),
                date_series
                    .as_date_iter()
                    .map(|value| value.map(|date| china::SSE.find_workday(date, kwargs.offset))),
            )
        }
    };
    Ok(res.into_series())
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
    let date_series = date_col.date()?;
    let res: BooleanChunked = match kwargs.market {
        Market::IB => date_series
            .as_date_iter()
            .map(|value| value.map(|dt| china::IB.is_business_day(dt)))
            .collect_trusted(),
        Market::SSE | Market::SH | Market::SZ | Market::SZE => date_series
            .as_date_iter()
            .map(|value| value.map(|dt| china::SSE.is_business_day(dt)))
            .collect_trusted(),
    };
    Ok(res.into_series())
}

fn batch_eval_neutral_net_basis_spread_impl(
    future: &StringChunked,
    bond: &StringChunked,
    date: &DateChunked,
    future_price: &Float64Chunked,
    bond_ytm: &Float64Chunked,
    capital_rate: &Float64Chunked,
    ctd_bond: &StringChunked,
    ctd_ytm: &Float64Chunked,
    reinvest_rate: Option<f64>,
) -> PolarsResult<Vec<Option<f64>>> {
    let reinvest_rate = Some(reinvest_rate.unwrap_or(0.0));
    let len_vec = [
        future_price.len(),
        bond_ytm.len(),
        bond.len(),
        future.len(),
        date.len(),
        ctd_bond.len(),
        ctd_ytm.len(),
    ];
    let len = *len_vec.iter().max().unwrap();
    if *len_vec.iter().min().unwrap() == 0 {
        return Ok(Default::default());
    }

    let mut future_iter = future.iter();
    let mut future_price_iter = future_price.iter();
    let mut bond_iter = bond.iter();
    let mut bond_ytm_iter = bond_ytm.iter();
    let mut capital_rate_iter = capital_rate.iter();
    let mut date_iter = date.as_date_iter();
    let mut ctd_bond_iter = ctd_bond.iter();
    let mut ctd_ytm_iter = ctd_ytm.iter();

    let mut result = Vec::with_capacity(len);
    let mut future: Arc<Future> = Future::new(future_iter.next().unwrap().unwrap_or("")).into();
    let mut future_price = future_price_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut bond =
        CachedBond::new(bond_iter.next().unwrap().unwrap_or(""), None).unwrap_or_default();
    let mut bond_ytm = bond_ytm_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut date = date_iter.next().unwrap().unwrap_or_default();
    let mut capital_rate = capital_rate_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut ctd_bond_cached =
        CachedBond::new(ctd_bond_iter.next().unwrap().unwrap_or(""), None).unwrap_or_default();
    let mut ctd_ytm_val = ctd_ytm_iter.next().unwrap().unwrap_or(f64::NAN);

    let mut evaluator = TfEvaluator {
        date,
        future: (future.clone(), future_price).into(),
        bond: BondYtm::new(bond.clone(), bond_ytm),
        capital_rate,
        reinvest_rate,
        ..Default::default()
    };

    if evaluator.future.code.is_empty()
        || evaluator.bond.code().is_empty()
        || ctd_bond_cached.code().is_empty()
    {
        result.push(None);
    } else {
        let ctd = BondYtm::new(ctd_bond_cached.clone(), ctd_ytm_val);
        let value = evaluator
            .clone()
            .neutral_net_basis_spread(ctd)
            .ok()
            .filter(|v| !v.is_nan());
        result.push(value);
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
            date = dt.unwrap_or_default();
        };
        if let Some(f) = future_iter.next() {
            if let Some(f) = f {
                if future.code != f {
                    future = Future::new(f).into()
                }
            } else {
                result.push(None);
                bond_iter.next();
                ctd_bond_iter.next();
                ctd_ytm_iter.next();
                continue;
            }
        };
        if let Some(b) = bond_iter.next() {
            if let Some(b) = b {
                if b != bond.code() && bond.bond_code != b {
                    bond = CachedBond::new(b, None).unwrap_or_default();
                }
            } else {
                result.push(None);
                ctd_bond_iter.next();
                ctd_ytm_iter.next();
                continue;
            }
        };
        if let Some(ctd_b) = ctd_bond_iter.next() {
            if let Some(ctd_b) = ctd_b {
                if ctd_b != ctd_bond_cached.code() && ctd_bond_cached.bond_code != ctd_b {
                    ctd_bond_cached = CachedBond::new(ctd_b, None).unwrap_or_default();
                }
            } else {
                result.push(None);
                ctd_ytm_iter.next();
                continue;
            }
        };
        if let Some(cy) = ctd_ytm_iter.next() {
            ctd_ytm_val = cy.unwrap_or(f64::NAN);
        };

        evaluator = evaluator.update_with_new_info(
            date,
            (future.clone(), future_price),
            (bond.clone(), bond_ytm),
            capital_rate,
            reinvest_rate,
        );

        if evaluator.future.code.is_empty()
            || evaluator.bond.code().is_empty()
            || ctd_bond_cached.code().is_empty()
        {
            result.push(None);
            continue;
        }

        let ctd = BondYtm::new(ctd_bond_cached.clone(), ctd_ytm_val);
        let value = evaluator
            .clone()
            .neutral_net_basis_spread(ctd)
            .ok()
            .filter(|v| !v.is_nan());
        result.push(value);
    }
    Ok(result)
}

#[polars_expr(output_type=Float64)]
fn evaluators_neutral_net_basis_spread(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let (future, bond, date, future_price, bond_ytm, capital_rate, ctd_bond, ctd_ytm) = (
        &inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4], &inputs[5], &inputs[6],
        &inputs[7],
    );
    let (future_price, bond_ytm, capital_rate, ctd_ytm) =
        auto_cast!(Float64(future_price, bond_ytm, capital_rate, ctd_ytm));
    let date = auto_cast!(Date(date));
    let bond = auto_cast!(String(bond));
    let ctd_bond = auto_cast!(String(ctd_bond));

    let result: Float64Chunked = batch_eval_neutral_net_basis_spread_impl(
        future.str()?,
        bond.str()?,
        date.date()?,
        future_price.f64()?,
        bond_ytm.f64()?,
        capital_rate.f64()?,
        ctd_bond.str()?,
        ctd_ytm.f64()?,
        kwargs.reinvest_rate,
    )?
    .into_iter()
    .collect_trusted();
    Ok(result.into_series())
}
