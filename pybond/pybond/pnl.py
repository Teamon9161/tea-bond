from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import polars as pl
    from polars.type_aliases import IntoExpr

from .polars_utils import parse_into_expr, register_plugin


class Fee:
    """Represents a fee for a trade."""

    def __init__(self, fee_str: str = ""):
        self.str = fee_str

    def __add__(self, other: Fee) -> Fee:
        return Fee(self.str + "+" + other.str)

    def __radd__(self, other: Fee) -> Fee:
        return Fee(other.str + "+" + self.str)

    @staticmethod
    def trade(fee) -> TradeFee:
        return TradeFee(fee)

    @staticmethod
    def qty(fee) -> QtyFee:
        return QtyFee(fee)

    @staticmethod
    def percent(fee) -> PercentFee:
        return PercentFee(fee)


class TradeFee(Fee):
    """Represents a fixed fee for a trade."""

    def __init__(self, fee: float):
        self.str = f"Trade({fee})"


class QtyFee(Fee):
    """Represents a fee based on the quantity of a trade."""

    def __init__(self, fee: float):
        self.str = f"Qty({fee})"


class PercentFee(Fee):
    """Represents a fee based on a percentage of the trade amount."""

    def __init__(self, fee: float):
        self.str = f"Percent({fee})"


def calc_bond_trade_pnl(
    symbol: IntoExpr,
    settle_time: IntoExpr,
    qty: IntoExpr,
    clean_price: IntoExpr,
    clean_close: IntoExpr,
    # symbol: str = "",
    bond_info_path: str | None = None,
    multiplier: float = 1,
    fee: str | Fee = "",
    borrowing_cost: float = 0,
    capital_rate: float = 0,
    begin_state=None,
) -> pl.Expr:
    if isinstance(fee, Fee):
        fee = fee.str
    symbol = parse_into_expr(symbol)
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
            "fee": 0,
        }
    kwargs = {
        # "symbol": symbol,
        "multiplier": multiplier,
        "fee": fee,
        "borrowing_cost": borrowing_cost,
        "capital_rate": capital_rate,
        "begin_state": begin_state,
        "bond_info_path": bond_info_path,
    }
    return register_plugin(
        args=[symbol, settle_time, qty, clean_price, clean_close],
        kwargs=kwargs,
        symbol="calc_bond_trade_pnl",
        is_elementwise=False,
    )
