from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import polars as pl
    from polars.type_aliases import IntoExpr

from .polars_utils import parse_into_expr, register_plugin


def calc_bond_trade_pnl(
    settle_time: IntoExpr,
    qty: IntoExpr,
    clean_price: IntoExpr,
    clean_close: IntoExpr,
    symbol: str = "",
    bond_info_path: str | None = None,
    multiplier: float = 1,
    c_rate: float = 0,
    borrowing_cost: float = 0,
    capital_rate: float = 0,
    begin_state=None,
) -> pl.Expr:
    settle_time = parse_into_expr(settle_time)
    qty = parse_into_expr(qty)
    clean_price = parse_into_expr(clean_price)
    clean_close = parse_into_expr(clean_close)
    if bond_info_path is None:
        from .bond import bonds_info_path as path

        bond_info_path = str(path)

    if begin_state is None:
        begin_state = {
            "pos": 0,
            "avg_price": 0,
            "pnl": 0,
            "realized_pnl": 0,
            "pos_price": 0,
            "unrealized_pnl": 0,
            "coupon_paid": 0,
            "amt": 0,
        }
    kwargs = {
        "symbol": symbol,
        "multiplier": multiplier,
        "c_rate": c_rate,
        "borrowing_cost": borrowing_cost,
        "capital_rate": capital_rate,
        "begin_state": begin_state,
        "bond_info_path": bond_info_path,
    }
    return register_plugin(
        args=[settle_time, qty, clean_price, clean_close],
        kwargs=kwargs,
        symbol="calc_bond_trade_pnl",
        is_elementwise=False,
    )
