use pyo3_polars::derive::polars_expr;
use tea_bond::pnl::{self, BondTradePnlOpt, PnlReport};
use tevec::export::arrow as polars_arrow;
use tevec::export::polars::prelude::*;
use tevec::prelude::{IsNone, Vec1Collect};

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

#[allow(clippy::useless_conversion)] // needed for support polars version below 0.43
pub fn pnl_report_vec_to_series(reports: &[PnlReport]) -> Series {
    use tevec::export::polars::prelude::*;
    let pos: Float64Chunked = reports
        .iter()
        .map(|t| t.pos.to_opt())
        .collect_trusted_vec1();
    let avg_price: Float64Chunked = reports
        .iter()
        .map(|t| t.avg_price.to_opt())
        .collect_trusted_vec1();
    let pnl: Float64Chunked = reports
        .iter()
        .map(|t| t.pnl.to_opt())
        .collect_trusted_vec1();
    let realized_pnl: Float64Chunked = reports
        .iter()
        .map(|t| t.realized_pnl.to_opt())
        .collect_trusted_vec1();
    let pos_price: Float64Chunked = reports
        .iter()
        .map(|t| t.pos_price.to_opt())
        .collect_trusted_vec1();
    let unrealized_pnl: Float64Chunked = reports
        .iter()
        .map(|t| t.unrealized_pnl.to_opt())
        .collect_trusted_vec1();
    let coupon_paid: Float64Chunked = reports
        .iter()
        .map(|t| t.coupon_paid.to_opt())
        .collect_trusted_vec1();
    let amt: Float64Chunked = reports
        .iter()
        .map(|t| t.amt.to_opt())
        .collect_trusted_vec1();
    let fee: Float64Chunked = reports
        .iter()
        .map(|t| t.fee.to_opt())
        .collect_trusted_vec1();
    let res: StructChunked = StructChunked::from_series(
        "pnl_report".into(),
        pos.len(),
        [
            pos.into_series().with_name("pos".into()),
            avg_price.into_series().with_name("avg_price".into()),
            pnl.into_series().with_name("pnl".into()),
            realized_pnl.into_series().with_name("realized_pnl".into()),
            pos_price.into_series().with_name("pos_price".into()),
            unrealized_pnl
                .into_series()
                .with_name("unrealized_pnl".into()),
            coupon_paid.into_series().with_name("coupon_paid".into()),
            amt.into_series().with_name("amt".into()),
            fee.into_series().with_name("fee".into()),
        ]
        .iter(),
    )
    .unwrap();
    res.into_series()
}

fn get_pnl_output_type(_input_fields: &[Field]) -> PolarsResult<Field> {
    let dtype = DataType::Struct(vec![
        Field::new("pos".into(), DataType::Float64),
        Field::new("avg_price".into(), DataType::Float64),
        Field::new("pnl".into(), DataType::Float64),
        Field::new("realized_pnl".into(), DataType::Float64),
        Field::new("pos_price".into(), DataType::Float64),
        Field::new("unrealized_pnl".into(), DataType::Float64),
        Field::new("coupon_paid".into(), DataType::Float64),
        Field::new("amt".into(), DataType::Float64),
        Field::new("fee".into(), DataType::Float64),
    ]);
    Ok(Field::new("pnl_report".into(), dtype))
}

#[polars_expr(output_type_func=get_pnl_output_type)]
fn calc_bond_trade_pnl(inputs: &[Series], kwargs: BondTradePnlOpt) -> PolarsResult<Series> {
    let (symbol, time, qty, clean_price, clean_close) =
        (&inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4]);
    let symbol = auto_cast!(String(symbol));
    let symbol = if let Some(s) = symbol.str()?.iter().next() {
        s
    } else {
        return Ok(pnl_report_vec_to_series(&[]));
    };
    let (qty, clean_price, clean_close) = auto_cast!(Float64(qty, clean_price, clean_close));
    let time = match time.dtype() {
        DataType::Date => time.clone(),
        _ => time.cast(&DataType::Date)?,
    };
    let profit_vec = pnl::calc_bond_trade_pnl(
        symbol,
        time.date()?.physical(),
        qty.f64()?,
        clean_price.f64()?,
        clean_close.f64()?,
        &kwargs,
    );
    let out = pnl_report_vec_to_series(&profit_vec);
    Ok(out)
}

fn get_trading_output_type(input_fields: &[Field]) -> PolarsResult<Field> {
    let dtype = DataType::Struct(vec![
        Field::new("time".into(), input_fields[0].dtype().clone()),
        Field::new("price".into(), DataType::Float64),
        Field::new("qty".into(), DataType::Float64),
    ]);
    Ok(Field::new("pnl_report".into(), dtype))
}

#[polars_expr(output_type_func=get_trading_output_type)]
fn trading_from_pos(inputs: &[Series], mut kwargs: pnl::TradeFromPosOpt) -> PolarsResult<Series> {
    use pyo3_polars::export::polars_core::utils::CustomIterTools;
    use tevec::export::polars::prelude::*;
    let (time, pos, open, finish_price, cash) = (&inputs[0], &inputs[1], &inputs[2], &inputs[3], &inputs[4]);
    let (pos, open, finish_price, cash) = auto_cast!(Float64(pos, open, finish_price, cash));
    if let Some(p) = finish_price.f64()?.iter().next() {
        kwargs.finish_price = p
    };
    if let Some(c) = cash.f64()?.iter().next() {
        kwargs.cash = c
    };
    let res = match time.dtype() {
        DataType::Date => {
            let trade_vec =
                pnl::trading_from_pos(time.date()?.physical(), pos.f64()?, open.f64()?, &kwargs);
            let time: Int32Chunked = trade_vec.iter().map(|t| t.time).collect_trusted();
            let time = time.into_date().into_series();
            let price: Float64Chunked = trade_vec.iter().map(|t| Some(t.price)).collect_trusted();
            let price = price.into_series();
            let qty: Float64Chunked = trade_vec.iter().map(|t| Some(t.qty)).collect_trusted();
            StructChunked::from_series(
                "trade".into(),
                time.len(),
                [
                    time.with_name("time".into()),
                    price.into_series().with_name("price".into()),
                    qty.into_series().with_name("qty".into()),
                ]
                .iter(),
            )
            .unwrap()
            .into_series()
        }
        _ => {
            let time_ca = time.datetime()?;
            let time_unit = time_ca.time_unit();
            let time_zone = time_ca.time_zone();
            let trade_vec =
                pnl::trading_from_pos(time_ca.physical(), pos.f64()?, open.f64()?, &kwargs);
            let time: Int64Chunked = trade_vec.iter().map(|t| t.time).collect_trusted();
            let time = time
                .into_datetime(time_unit, time_zone.clone())
                .into_series();
            let price: Float64Chunked = trade_vec.iter().map(|t| Some(t.price)).collect_trusted();
            let price = price.into_series();
            let qty: Float64Chunked = trade_vec.iter().map(|t| Some(t.qty)).collect_trusted();
            StructChunked::from_series(
                "trade".into(),
                time.len(),
                [
                    time.with_name("time".into()),
                    price.into_series().with_name("price".into()),
                    qty.into_series().with_name("qty".into()),
                ]
                .iter(),
            )
            .unwrap()
            .into_series()
        }
    };
    Ok(res)
}
