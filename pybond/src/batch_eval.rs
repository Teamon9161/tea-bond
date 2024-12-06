use std::sync::Arc;

use super::bond::PyBond;
use super::future::PyFuture;
use super::tf_evaluator::PyTfEvaluator;
use chrono::NaiveDate;
use pyo3::prelude::*;
use tea_bond::{Bond, BondYtm, Future, FuturePrice, TfEvaluator};
use tevec::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Evaluators(pub Vec<TfEvaluator>);

impl From<Vec<TfEvaluator>> for Evaluators {
    #[inline]
    fn from(value: Vec<TfEvaluator>) -> Self {
        Self(value)
    }
}

fn get_bond_ytm_future_price<T: IsNone<Inner = f64>>(
    ytm: T,
    price: T,
    bond: Arc<Bond>,
    future: Arc<Future>,
) -> (BondYtm, FuturePrice) {
    let ytm = if ytm.is_none() {
        f64::NAN
    } else {
        ytm.unwrap()
    };
    let price = if price.is_none() {
        f64::NAN
    } else {
        price.unwrap()
    };
    (BondYtm { bond, ytm }, FuturePrice { future, price })
}

fn evaluate_price_vec<V: Vec1View<T>, T: IsNone<Inner = f64>>(
    evaluator: TfEvaluator,
    bond_ytm: V,
    future_price: V,
    bond: Arc<Bond>,
    future: Arc<Future>,
    date: NaiveDate,
    capital_rate: f64,
    reinvest_rate: Option<f64>,
    update: bool,
) -> Vec<TfEvaluator> {
    if update {
        bond_ytm
            .titer()
            .zip(future_price.titer())
            .map(|(ytm, price)| {
                let (bond_ytm, future_price) =
                    get_bond_ytm_future_price(ytm, price, bond.clone(), future.clone());
                evaluator.clone().update_with_new_info(
                    date,
                    future_price,
                    bond_ytm,
                    capital_rate,
                    reinvest_rate,
                )
            })
            .collect_trusted_to_vec()
    } else {
        bond_ytm
            .titer()
            .zip(future_price.titer())
            .map(|(ytm, price)| {
                let (bond, future) =
                    get_bond_ytm_future_price(ytm, price, bond.clone(), future.clone());
                TfEvaluator {
                    date,
                    bond,
                    future,
                    capital_rate,
                    reinvest_rate,
                    ..Default::default()
                }
            })
            .collect_trusted_to_vec()
    }
}

#[pymethods]
impl PyTfEvaluator {
    #[cfg(feature = "numpy")]
    #[pyo3(signature = (bond_ytm, future_price, bond=None, future=None, date=None, capital_rate=None, reinvest_rate=None, update=false))]
    pub fn evaluate_price_vec(
        &self,
        bond_ytm: &Bound<'_, PyAny>,
        future_price: &Bound<'_, PyAny>,
        bond: Option<PyBond>,
        future: Option<PyFuture>,
        date: Option<&Bound<'_, PyAny>>,
        capital_rate: Option<f64>,
        mut reinvest_rate: Option<f64>,
        update: bool,
    ) -> PyResult<Evaluators> {
        use numpy::{PyArray1, PyArrayMethods};
        let bond_ytm = bond_ytm.downcast::<PyArray1<f64>>()?.try_readonly()?;
        let future_price = future_price.downcast::<PyArray1<f64>>()?.try_readonly()?;
        let bond_ytm_read = bond_ytm.as_array();
        let future_price_read = future_price.as_array();
        let bond = bond
            .map(|b| b.0)
            .unwrap_or_else(|| self.0.bond.bond.clone());
        let future = future
            .map(|f| f.0)
            .unwrap_or_else(|| self.0.future.future.clone());
        let date = if let Some(date) = date {
            date.extract::<NaiveDate>()?
        } else {
            self.0.date
        };
        let capital_rate = capital_rate.unwrap_or(self.0.capital_rate);
        if reinvest_rate.is_none() {
            if let Some(base_reinvest_rate) = self.0.reinvest_rate {
                reinvest_rate = Some(base_reinvest_rate);
            };
        }
        let evaluators = evaluate_price_vec(
            self.0.clone(),
            bond_ytm_read,
            future_price_read,
            bond,
            future,
            date,
            capital_rate,
            reinvest_rate,
            update,
        );
        Ok(evaluators.into())
    }
}
