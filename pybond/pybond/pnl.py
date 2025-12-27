from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

import polars as pl

if TYPE_CHECKING:
    from polars._typing import IntoExpr

from .polars_utils import parse_into_expr, register_plugin


class Fee(pl.Expr):
    def __new__(cls, fee_dict=None):
        if fee_dict is None:
            fee_dict = {"kind": "zero"}
        obj = super().__new__(cls)
        obj._fee_dict = fee_dict
        return obj

    @property
    def kind(self):
        return self._fee_dict.get("kind")

    @property
    def fee(self):
        return self._fee_dict.get("fee")

    @property
    def _pyexpr(self):
        return pl.lit(json.dumps(self._fee_dict))._pyexpr

    def __add__(self, other: Fee | int | float):
        left_items = self._fee_dict["items"] if self.kind == "sum" else [self._fee_dict]
        if isinstance(other, Fee):
            right_items = (
                other._fee_dict["items"] if other.kind == "sum" else [other._fee_dict]
            )
            return Fee({"kind": "sum", "items": left_items + right_items})
        elif isinstance(other, (int, float)) and self.kind in [
            "fixed",
            "per_trade",
            "per_qty",
            "percent",
        ]:
            _dict = self._fee_dict.copy()
            _dict["fee"] += other
            return Fee(_dict)
        else:
            msg = f"Cannot add {other} on {self.kind} Fee"
            raise NotImplementedError(msg)

    def __radd__(self, other):
        return self.__add__(other)

    def min(self, fee: float):
        """
        fee = min(cap, inner_fee)
        """
        return Fee({"kind": "min", "fee": self._fee_dict, "cap": fee})

    def max(self, fee: float):
        """
        fee = max(floor, inner_fee)
        """
        return Fee({"kind": "max", "fee": self._fee_dict, "floor": fee})

    @staticmethod
    def zero():
        """Represents a fee of zero."""
        return Fee({"kind": "zero"})

    @staticmethod
    def fixed(fee: float) -> TradeFee:
        """Represents a fixed fee for a trade."""
        return Fee({"kind": "fixed", "fee": fee})

    @staticmethod
    def trade(fee: float) -> TradeFee:
        """Represents a fixed fee for a trade."""
        return Fee({"kind": "per_trade", "fee": fee})

    @staticmethod
    def qty(fee: float) -> QtyFee:
        """Represents a fee based on the quantity of a trade."""
        return Fee({"kind": "per_qty", "fee": fee})

    @staticmethod
    def percent(fee: float) -> PercentFee:
        """Represents a fee based on a percentage of the trade amount."""
        return Fee({"kind": "percent", "fee": fee})


class FeeZero(Fee):
    def __new__(cls):
        obj = super().__new__(cls)
        obj._fee_dict = Fee.zero()._fee_dict
        return obj


class PercentFee(Fee):
    """Represents a fee based on a percentage of the trade amount."""

    def __new__(cls, rate: float):
        obj = super().__new__(cls)
        obj._fee_dict = Fee.percent(rate)._fee_dict
        return obj


class QtyFee(Fee):
    """Represents a fee based on the quantity of a trade."""

    def __new__(cls, per_qty: float):
        obj = super().__new__(cls)
        obj._fee_dict = Fee.qty(per_qty)._fee_dict
        return obj


class TradeFee(Fee):
    """Represents a fixed fee for a trade."""

    def __new__(cls, per_trade: float):
        obj = super().__new__(cls)
        obj._fee_dict = Fee.trade(per_trade)._fee_dict
        return obj


def calc_bond_trade_pnl(
    symbol: IntoExpr,
    settle_time: IntoExpr,
    qty: IntoExpr | None = None,
    clean_price: IntoExpr | None = None,
    clean_close: IntoExpr = "close",
    bond_info_path: str | None = None,
    multiplier: IntoExpr | None = None,
    fee: IntoExpr | Fee | None = None,
    begin_state: IntoExpr | None = None,
) -> pl.Expr:
    """
    计算债券交易pnl
    symbol: 交易的标的名称, 如果不是债券传⼊空字符串即可。
    settle_time: 结算时间,
        如果settle_time传入代表Trade的struct Series(包含time, price, qty三个field), 则可以不传qty和clean_price
    qty: 成交量, 正负号表⽰⽅向
    clean_price: 成交的净价
    clean_close: 当前时间段的最新价格(净价)
    bond_info_path: 可以指定债券信息的存放⽂件夹, 不传⼊则使⽤默认路径.
    multiplier: 合约乘数, 例如对于债券, 1000的成交对应1000w, 合约乘数应为100, 默认为1
    fee: 交易费⽤
    费⽤设置说明:
        TradeFee: 每笔成交⽀付的费⽤
        QtyFee: 每⼿需要⽀付的费⽤
        PercentFee: 按照成交⾦额百分⽐⽀付的费⽤
        费⽤⽀持相加, 例如 QtyFee(120) + TradeFee(20)
    """
    assert clean_close is not None
    if fee is None:
        fee = Fee.zero()
    fee = parse_into_expr(fee)
    symbol = parse_into_expr(symbol)
    settle_time = parse_into_expr(settle_time)
    clean_close = parse_into_expr(clean_close)
    multiplier = parse_into_expr(multiplier)
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
    kwargs = {"bond_info_path": bond_info_path}
    if all(x is None for x in [qty, clean_price]):
        # struct settle_time, contains trade info
        args = [symbol, settle_time, clean_close, begin_state, multiplier, fee]
    else:
        qty = parse_into_expr(qty)
        clean_price = parse_into_expr(clean_price)
        args = [
            symbol,
            settle_time,
            qty,
            clean_price,
            clean_close,
            begin_state,
            multiplier,
            fee,
        ]
    return register_plugin(
        args=args,
        kwargs=kwargs,
        symbol="calc_bond_trade_pnl",
        is_elementwise=False,
    )


def calc_trade_pnl(
    time: IntoExpr,
    qty: IntoExpr | None = None,
    price: IntoExpr | None = None,
    close: IntoExpr = "close",
    multiplier: IntoExpr | None = None,
    fee: IntoExpr | Fee | None = None,
    begin_state: IntoExpr | None = None,
):
    """
    计算交易pnl
    symbol: 交易的标的名称, 如果不是债券传⼊空字符串即可。
    time: 交易时间,
        如果time传入代表Trade的struct Series(包含time, price, qty三个field), 则可以不传qty和clean_price
    qty: 成交量, 正负号表⽰⽅向
    clean_price: 成交的净价
    clean_close: 当前时间段的最新价格(净价)
    multiplier: 合约乘数, 例如对于债券, 1000的成交对应1000w, 合约乘数应为100, 默认为1
    fee: 交易费⽤
    费⽤设置说明:
        TradeFee: 每笔成交⽀付的费⽤
        QtyFee: 每⼿需要⽀付的费⽤
        PercentFee: 按照成交⾦额百分⽐⽀付的费⽤
        费⽤⽀持相加, 例如 QtyFee(120) + TradeFee(20)
    """
    return calc_bond_trade_pnl(
        symbol=pl.lit(""),
        settle_time=time,
        qty=qty,
        clean_price=price,
        clean_close=close,
        multiplier=multiplier,
        fee=fee,
        begin_state=begin_state,
    )


def trading_from_pos(
    time: IntoExpr,
    pos: IntoExpr,
    open: IntoExpr,
    finish_price: IntoExpr | None = None,
    cash: IntoExpr = 1e8,
    multiplier: IntoExpr | None = None,
    qty_tick: IntoExpr = 1.0,
    min_adjust_amt: float = 0.0,
    *,
    stop_on_finish: bool = False,
    keep_shape: bool = False,
) -> pl.Expr:
    """
    生成交易记录
    time: ⽤于⽣成成交时间, ⽀持任意可以转为polars表达式的输⼊
    pos: 当前时间的实际仓位, -1 ~ 1, 表⽰百分⽐
    open: 当前周期的开仓价格
    cash: 总资⾦, ⽤于计算实际开仓⼿数
    multiplier: 合约乘数, 默认为1
    qty_tick: 最⼩开仓⼿数, 例如0.01, 0.1, 1, 100
    stop_on_finish: 当前标的没有数据后是否平仓
    finish_price: 当前标的没数据时的平仓价格, ⽀持polars表达式
    keep_shape: 是否维持表达式的长度, 不保留则只返回实际发生的交易
    """
    time = parse_into_expr(time)
    pos = parse_into_expr(pos)
    open = parse_into_expr(open)
    multiplier = parse_into_expr(multiplier)
    qty_tick = parse_into_expr(qty_tick)
    if finish_price is not None:
        stop_on_finish = True
    finish_price = parse_into_expr(finish_price)
    cash = parse_into_expr(cash)
    kwargs = {
        "cash": None,  # 会从表达式中获取
        "multiplier": 0.0,  # 会从表达式中获取
        "qty_tick": 1.0,  # 会从表达式中获取
        "stop_on_finish": stop_on_finish,
        "finish_price": None,  # 会从表达式中获取
        "min_adjust_amt": float(min_adjust_amt),
        "keep_shape": bool(keep_shape),
    }
    return register_plugin(
        args=[time, pos, open, finish_price, cash, multiplier, qty_tick],
        kwargs=kwargs,
        symbol="trading_from_pos",
        is_elementwise=False,
    )
