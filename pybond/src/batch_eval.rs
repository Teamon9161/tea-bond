use crate::ffi::get_bond_data_path;
use chrono::{Days, NaiveDate};
use pyo3_polars::derive::polars_expr;
use pyo3_polars::export::polars_core::utils::CustomIterTools;
use serde::Deserialize;
use tea_bond::{CachedBond, Future, TfEvaluator};
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
            $se.clone()
        } else {
            $se.cast(&DataType::$arm)?
        }
    };
    // for multiple expressions
    ($arm: ident ($($se: expr),*)) => {
        ($(
            if let DataType::$arm = $se.dtype() {
                $se.clone()
            } else {
                $se.cast(&DataType::$arm)?
            }
        ),*)
    };
}

pub const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

fn calc_nbs(
    future: &StringChunked,
    bond: &StringChunked,
    date: &DateChunked,
    future_price: &Float64Chunked,
    bond_ytm: &Float64Chunked,
    capital_rate: &Float64Chunked,
    reinvest_rate: Option<f64>,
) -> Float64Chunked {
    let reinvest_rate = Some(reinvest_rate.unwrap_or(0.0));
    let bond_data_path = get_bond_data_path();
    let path = bond_data_path.as_deref();
    // let evaluator = TfEvaluator::default();
    let len = future_price.len();
    if len == 0 {
        return Default::default();
    }
    // get iterators
    let mut future_iter = future.iter();
    let mut future_price_iter = future_price.iter();
    let mut bond_iter = bond.iter();
    let mut bond_ytm_iter = bond_ytm.iter();
    let mut capital_rate_iter = capital_rate.iter();
    let mut date_iter = date.physical().iter();
    // create result vector
    let mut result = Vec::with_capacity(len);
    // create firsth TfEvaluator
    let mut future: Arc<Future> = Future::new(future_iter.next().unwrap().unwrap()).into();
    let mut future_price = future_price_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut bond = CachedBond::new(bond_iter.next().unwrap().unwrap(), path).unwrap();
    let mut bond_ytm = bond_ytm_iter.next().unwrap().unwrap_or(f64::NAN);
    let mut date_physical = date_iter.next().unwrap().unwrap_or(0);
    let mut date = EPOCH
        .checked_add_days(Days::new(date_physical as u64))
        .unwrap();
    let mut capital_rate = capital_rate_iter.next().unwrap().unwrap();
    let mut evaluator = TfEvaluator {
        date,
        future: (future.clone(), future_price).into(),
        bond: (bond.clone(), bond_ytm).into(),
        capital_rate,
        reinvest_rate,
        ..Default::default()
    };
    evaluator = evaluator.with_net_basis_spread().unwrap();
    result.push(evaluator.net_basis_spread.unwrap());
    for _ in 1..len {
        if let Some(f) = future_iter.next() {
            if let Some(f) = f {
                if f != &future.code {
                    future = Future::new(f).into()
                }
            } else {
                // TODO(Teamon): 期货如果为空，可能影响结果正确性，最好有进一步的处理
                future = Default::default();
            }
        };
        if let Some(b) = bond_iter.next() {
            if let Some(b) = b {
                if b != bond.code() && b != &bond.bond_code {
                    bond = CachedBond::new(b, path).unwrap();
                }
            } else {
                bond = Default::default();
            }
        };
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
        evaluator = evaluator.update_with_new_info(
            date,
            (future.clone(), future_price),
            (bond.clone(), bond_ytm),
            capital_rate,
            reinvest_rate,
        );
        evaluator = evaluator.with_net_basis_spread().unwrap();
        result.push(evaluator.net_basis_spread.unwrap())
    }
    result
        .into_iter()
        .map(|v| if v.is_nan() { None } else { Some(v) })
        .collect_trusted()
}

#[polars_expr(output_type=Float64)]
fn evaluators_net_basis_spread(
    inputs: &[Series],
    kwargs: EvaluatorBatchParams,
) -> PolarsResult<Series> {
    let (future, bond, date, future_price, bond_ytm, capital_rate) = (
        &inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4], &inputs[5],
    );
    let (future_price, bond_ytm, capital_rate) =
        auto_cast!(Float64(future_price, bond_ytm, capital_rate));
    let date = match date.dtype() {
        DataType::Date => date.clone(),
        _ => date.cast(&DataType::Date)?,
    };
    let result = calc_nbs(
        future.str()?,
        bond.str()?,
        date.date()?,
        future_price.f64()?,
        bond_ytm.f64()?,
        capital_rate.f64()?,
        kwargs.reinvest_rate,
    );
    Ok(result.into_series())
}
