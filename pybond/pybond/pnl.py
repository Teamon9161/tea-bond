from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

import polars as pl

if TYPE_CHECKING:
    from polars.type_aliases import IntoExpr

from .polars_utils import parse_into_expr, register_plugin


class Fee:
    def to_dict(self) -> dict[str, Any]:
        raise NotImplementedError

    def __add__(self, other: Fee) -> FeeSum:
        if isinstance(self, FeeSum):
            if isinstance(other, FeeSum):
                return FeeSum(items=self.items + other.items)
            return FeeSum(items=[*self.items, other])
        else:
            if isinstance(other, FeeSum):
                return FeeSum(items=[self, *other.items])
            return FeeSum(items=[self, other])

    @staticmethod
    def trade(fee) -> TradeFee:
        return TradeFee(fee)

    @staticmethod
    def qty(fee) -> QtyFee:
        return QtyFee(fee)

    @staticmethod
    def percent(fee) -> PercentFee:
        return PercentFee(fee)


@dataclass
class FeeZero(Fee):
    def to_dict(self):
        return {"kind": "zero"}


@dataclass
class PercentFee(Fee):
    """Represents a fee based on a percentage of the trade amount."""

    rate: float

    def to_dict(self):
        return {"kind": "percent", "rate": self.rate}


@dataclass
class QtyFee(Fee):
    """Represents a fee based on the quantity of a trade."""

    per_qty: float

    def to_dict(self):
        return {"kind": "per_qty", "per_qty": self.per_qty}


@dataclass
class TradeFee(Fee):
    """Represents a fixed fee for a trade."""

    per_trade: float

    def to_dict(self):
        return {"kind": "per_trade", "per_trade": self.per_trade}


@dataclass
class FeeSum(Fee):
    items: list[Fee] = field(default_factory=list)

    def to_dict(self):
        return {
            "kind": "sum",
            "items": [f.to_dict() for f in self.items],
        }


@dataclass
class MinFee(Fee):
    """Represents a minimum fee for a trade."""

    cap: float
    fee: Fee

    def to_dict(self):
        return {"kind": "min", "cap": self.cap, "fee": self.fee.to_dict()}


@dataclass
class MaxFee(Fee):
    """Represents a maximum fee for a trade."""

    floor: float
    fee: Fee

    def to_dict(self):
        return {"kind": "max", "floor": self.floor, "fee": self.fee.to_dict()}


def calc_bond_trade_pnl(
    symbol: IntoExpr,
    settle_time: IntoExpr,
    qty: IntoExpr,
    clean_price: IntoExpr,
    clean_close: IntoExpr,
    bond_info_path: str | None = None,
    multiplier: float = 1,
    fee: Fee | None = None,
    borrowing_cost: float = 0,
    capital_rate: float = 0,
    begin_state: IntoExpr | None = None,
) -> pl.Expr:
    if fee is None:
        fee = FeeZero()
    fee = fee.to_dict()
    symbol = parse_into_expr(symbol)
    settle_time = parse_into_expr(settle_time)
    qty = parse_into_expr(qty)
    clean_price = parse_into_expr(clean_price)
    clean_close = parse_into_expr(clean_close)
    if begin_state is not None and not isinstance(begin_state, dict):
        begin_state = parse_into_expr(begin_state)
    if bond_info_path is None:
        from .bond import bonds_info_path as path

        bond_info_path = str(path)

    if begin_state is None:
        begin_state = pl.lit(
            {
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
        )
    kwargs = {
        "multiplier": multiplier,
        "fee": fee,
        "borrowing_cost": borrowing_cost,
        "capital_rate": capital_rate,
        "bond_info_path": bond_info_path,
    }
    return register_plugin(
        args=[symbol, settle_time, qty, clean_price, clean_close, begin_state],
        kwargs=kwargs,
        symbol="calc_bond_trade_pnl",
        is_elementwise=False,
    )


def trading_from_pos(
    time: IntoExpr,
    pos: IntoExpr,
    open: IntoExpr,
    finish_price: IntoExpr | None = None,
    cash: IntoExpr = 1e8,
    multiplier: float = 1,
    qty_tick: float = 1.0,
    min_adjust_amt: float = 0.0,
    *,
    stop_on_finish: bool = False,
) -> pl.Expr:
    time = parse_into_expr(time)
    pos = parse_into_expr(pos)
    open = parse_into_expr(open)
    finish_price = parse_into_expr(finish_price)
    cash = parse_into_expr(cash)
    kwargs = {
        "cash": None,
        "multiplier": float(multiplier),
        "qty_tick": float(qty_tick),
        "stop_on_finish": stop_on_finish,
        "finish_price": None,
        "min_adjust_amt": float(min_adjust_amt),
    }
    return register_plugin(
        args=[time, pos, open, finish_price, cash],
        kwargs=kwargs,
        symbol="trading_from_pos",
        is_elementwise=False,
    )
