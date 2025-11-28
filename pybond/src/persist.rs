use std::borrow::Cow;

use chrono::NaiveDate;
use itertools::izip;
use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_polars::PyDataFrame;
use tea_bond::{Bond, WindSqlRow};
use tevec::export::polars::prelude::*;

macro_rules! auto_cast {
    // for one expression
    ($arm: ident ($se: expr)) => {
        if let DataType::$arm = $se.dtype() {
            $se
        } else {
            &$se.cast(&DataType::$arm).unwrap()
        }
    };
    // for multiple expressions
    ($arm: ident ($($se: expr),*)) => {
        ($(
            if let DataType::$arm = $se.dtype() {
                $se
            } else {
                &$se.cast(&DataType::$arm).unwrap()
            }
        ),*)
    };
}

#[pyfunction]
pub fn update_info_from_wind_sql_df(df: PyDataFrame) -> PyResult<()> {
    // use tevec::export::polars::prelude::Series;
    let df = df.0;
    let height = df.height();

    let col = |name: &str| -> &Column { df.column(name).unwrap() };

    // 自动类型对齐
    let s_windcode = auto_cast!(String(col("s_info_windcode")));
    let s_name = auto_cast!(String(col("s_info_name")));
    let b_par = auto_cast!(Float64(col("b_info_par")));
    let b_coupon = auto_cast!(Int32(col("b_info_coupon")));
    let b_interesttype = auto_cast!(Int32(col("b_info_interesttype")));
    let b_couponrate = auto_cast!(Float64(col("b_info_couponrate")));
    let b_spread = auto_cast!(Float64(col("b_info_spread")));
    let b_interestfrequency = auto_cast!(String(col("b_info_interestfrequency")));
    let b_carrydate = auto_cast!(String(col("b_info_carrydate")));
    let b_maturitydate = auto_cast!(String(col("b_info_maturitydate")));
    let b_referyield = auto_cast!(Float64(col("b_tendrst_referyield")));
    let b_issueprice = auto_cast!(Float64(col("b_info_issueprice")));

    let iter = izip!(
        s_windcode.str().unwrap(),
        s_name.str().unwrap(),
        b_par.f64().unwrap(),
        b_coupon.i32().unwrap(),
        b_interesttype.i32().unwrap(),
        b_couponrate.f64().unwrap(),
        b_spread.f64().unwrap(),
        b_interestfrequency.str().unwrap(),
        b_carrydate.str().unwrap(),
        b_maturitydate.str().unwrap(),
        b_referyield.f64().unwrap(),
        b_issueprice.f64().unwrap(),
    );

    for (
        idx,
        (
            windcode,
            name,
            par,
            coupon,
            interest_type,
            coupon_rate,
            spread,
            interest_freq,
            carry_date,
            maturity_date,
            refer_yield,
            issue_price,
        ),
    ) in iter.enumerate()
    {
        let windcode = windcode
            .map(Cow::Borrowed)
            .ok_or_else(|| PyValueError::new_err("s_info_windcode is required"))?;
        let name = name
            .map(Cow::Borrowed)
            .ok_or_else(|| PyValueError::new_err("s_info_name is required"))?;
        let par = par.ok_or_else(|| PyValueError::new_err("b_info_par is required"))?;
        let coupon = coupon.unwrap_or(505001000);

        let carry_date = carry_date
            .map(|d| NaiveDate::parse_from_str(d, "%Y%m%d").unwrap())
            .unwrap_or_default();
        let maturity_date = maturity_date
            .map(|d| NaiveDate::parse_from_str(d, "%Y%m%d").unwrap())
            .unwrap_or_default();

        let row = WindSqlRow {
            s_info_windcode: windcode.into_owned().into(),
            s_info_name: name.into_owned().into(),
            b_info_par: par,
            b_info_coupon: coupon,
            b_info_interesttype: interest_type,
            b_info_couponrate: coupon_rate,
            b_info_spread: spread,
            b_info_interestfrequency: interest_freq.map(|s| s.to_string().into()),
            b_info_carrydate: carry_date,
            b_info_maturitydate: maturity_date,
            b_tendrst_referyield: refer_yield,
            b_info_issueprice: issue_price,
        };

        let bond: Bond = row
            .try_into()
            .map_err(|e: anyhow::Error| PyValueError::new_err(e.to_string()))?;
        // 最后一行 flush_all=true
        bond.save_disk(idx + 1 == height)
            .map_err(|e: anyhow::Error| PyValueError::new_err(e.to_string()))?;
    }

    Ok(())
}
