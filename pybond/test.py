import os
from datetime import date

import polars as pl
from polars import col

# from pybond.pnl import calc_bond_trade_pnl
from pybond import Bond

Bond(250205)


def parse_into_expr(
    expr,
    *,
    str_as_lit: bool = False,
    list_as_lit: bool = True,
    dtype=None,
) -> pl.Expr:
    """
    Parse a single input into an expression.

    Parameters
    ----------
    expr
        The input to be parsed as an expression.
    str_as_lit
        Interpret string input as a string literal. If set to `False` (default),
        strings are parsed as column names.
    list_as_lit
        Interpret list input as a lit literal, If set to `False`,
        lists are parsed as `Series` literals.
    dtype
        If the input is expected to resolve to a literal with a known dtype, pass
        this to the `lit` constructor.

    Returns
    -------
    polars.Expr
    """
    if isinstance(expr, pl.Expr):
        pass
    elif isinstance(expr, str) and not str_as_lit:
        expr = pl.col(expr)
    elif isinstance(expr, list) and not list_as_lit:
        expr = pl.lit(pl.Series(expr), dtype=dtype)
    else:
        expr = pl.lit(expr, dtype=dtype)

    return expr


@pl.api.register_dataframe_namespace("trade")
@pl.api.register_lazyframe_namespace("trade")
class PLDFTradeExtend:
    def __init__(self, df):
        self.df = df

    def calc_bond_daily_pnl(
        self,
        settle_date="settle_date",
        symbol="symbol",
        qty="qty",
        clean_price="clean_price",
        clean_close="clean_close",
        date="date",
        direction=None,
        step=None,
        bond_multipliter=100,
        future_multiplier=10000,
    ):
        from pybond.pnl import calc_bond_trade_pnl

        df = self.df.collect() if isinstance(self.df, pl.LazyFrame) else self.df
        schema = df.schema
        value_exprs = []
        for out_name, value in [
            ("clean_price", clean_price),
            ("clean_close", clean_close),
        ]:
            if value is None:
                value = out_name
            value_name = value if isinstance(value, str) else value.meta.output_name()
            if value_name in schema:
                value_exprs.append(parse_into_expr(value).alias(out_name))
        if step is None:
            step = 1
        value_exprs.append(parse_into_expr(step).alias("step"))
        if settle_date is None:
            settle_date = parse_into_expr(date).alias("settle_date")
        if direction is not None:
            qty = parse_into_expr(qty) * parse_into_expr(direction)
        df = self.df.select(
            *value_exprs,
            date=date,
            symbol=symbol,
            qty=qty,
            settle_date=settle_date,
        )
        symbol_dfs = []
        for symbol, symbol_df in df.group_by(["symbol", "step"]):
            symbol = symbol[0]
            if symbol.startswith("T"):
                # 期货不需要计算付息
                _symbol = ""
                multiplier = future_multiplier
            else:
                _symbol = symbol
                multiplier = bond_multipliter
            symbol_df = (
                symbol_df.select(
                    [
                        "date",
                        "symbol",
                        calc_bond_trade_pnl(
                            settle_time="settle_date",
                            qty="qty",
                            clean_price="clean_price",
                            clean_close="clean_close",
                            multiplier=multiplier,
                            symbol=_symbol,
                        ).alias("qty"),
                    ]
                )
                .unnest("qty")
                .with_columns(pnl_chg=col("pnl") - col("pnl").shift().fill_null(0))
            )
            symbol_dfs.append(symbol_df)
        symbol_dfs = pl.concat(symbol_dfs).sort("date", "symbol")
        return symbol_dfs


# os.environ["POLARS_VERBOSE"] = "1"
# b = Bond(240215)
df = (
    pl.DataFrame(
        {
            "settle_time": [date(2025, 7, 3)] * 3
            + [date(2025, 7, 4)]
            + [date(2025, 7, 8)] * 2,
            "clean_price": [100, 101, 102, 200, 110, 115],
            "clean_close": [103, 103, 103, 200, 118, 118],
            "qty": [1, 1, 1, 1, 1, 1],
            "symbol": ["240215"] * 3 + ["250205"] + ["240215"] * 2,
        }
    )
    # .filter(col("symbol") == "240215")
    .trade.calc_bond_daily_pnl(settle_date="settle_time", date="settle_time")
)
df
# df.select(
#     calc_bond_trade_pnl(
#         "settle_time",
#         "qty",
#         "clean_price",
#         "clean_close",
#         symbol="240215",
#     ).struct.unnest()
# )
