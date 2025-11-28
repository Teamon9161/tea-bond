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
    def trade(fee: float) -> TradeFee:
        return TradeFee(fee)

    @staticmethod
    def qty(fee: float) -> QtyFee:
        return QtyFee(fee)

    @staticmethod
    def percent(fee: float) -> PercentFee:
        return PercentFee(fee)

    def zero(self) -> FeeZero:
        return FeeZero()

    def min(self, fee: float) -> MinFee:
        return MinFee(fee, self)

    def max(self, fee: float) -> MaxFee:
        return MaxFee(fee, self)


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
    qty: IntoExpr | None = None,
    clean_price: IntoExpr | None = None,
    clean_close: IntoExpr = "close",
    bond_info_path: str | None = None,
    multiplier: float = 1,
    fee: Fee | None = None,
    borrowing_cost: float = 0,
    capital_rate: float = 0,
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
    multiplier: 合约乘数, 例如对于债券, 1000的成交对应1000w, 合约乘数应为100
    fee: 交易费⽤
    费⽤设置说明:
        TradeFee: 每笔成交⽀付的费⽤
        QtyFee: 每⼿需要⽀付的费⽤
        PercentFee: 按照成交⾦额百分⽐⽀付的费⽤
        费⽤⽀持相加, 例如 QtyFee(120) + TradeFee(20)
    """
    assert clean_close is not None
    if fee is None:
        fee = FeeZero()
    fee = fee.to_dict()
    symbol = parse_into_expr(symbol)
    settle_time = parse_into_expr(settle_time)
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
    if all(x is None for x in [qty, clean_price]):
        # struct settle_time, contains trade info
        args = [symbol, settle_time, clean_close, begin_state]
    else:
        qty = parse_into_expr(qty)
        clean_price = parse_into_expr(clean_price)
        args = [symbol, settle_time, qty, clean_price, clean_close, begin_state]
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
    multiplier: float = 1,
    fee: Fee | None = None,
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
    multiplier: 合约乘数, 例如对于债券, 1000的成交对应1000w, 合约乘数应为100
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
    multiplier: float = 1,
    qty_tick: float = 1.0,
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
    multiplier: 合约乘数
    qty_tick: 最⼩开仓⼿数, 例如0.01, 0.1, 1, 100
    stop_on_finish: 当前标的没有数据后是否平仓
    finish_price: 当前标的没数据时的平仓价格, ⽀持polars表达式
    keep_shape: 是否维持表达式的长度, 不保留则只返回实际发生的交易
    """
    time = parse_into_expr(time)
    pos = parse_into_expr(pos)
    open = parse_into_expr(open)
    finish_price = parse_into_expr(finish_price)
    cash = parse_into_expr(cash)
    kwargs = {
        "cash": None,  # 会从表达式中获取
        "multiplier": float(multiplier),
        "qty_tick": float(qty_tick),
        "stop_on_finish": stop_on_finish,
        "finish_price": None,  # 会从表达式中获取
        "min_adjust_amt": float(min_adjust_amt),
        "keep_shape": bool(keep_shape),
    }
    return register_plugin(
        args=[time, pos, open, finish_price, cash],
        kwargs=kwargs,
        symbol="trading_from_pos",
        is_elementwise=False,
    )
