from __future__ import annotations

from typing import TYPE_CHECKING

from .pybond import Ib, Sse

if TYPE_CHECKING:
    from polars.type_aliases import IntoExpr
import polars as pl

from .polars_utils import parse_into_expr, register_plugin


class TfEvaluators:
    """A class for treasury futures evaluation using Polars expressions."""

    def __init__(
        self,
        future: IntoExpr = "future",
        bond: IntoExpr = "bond",
        date: IntoExpr = "date",
        future_price: IntoExpr = None,
        bond_ytm: IntoExpr = None,
        capital_rate: IntoExpr = None,
        reinvest_rate=None,
    ):
        """
        Initialize TfEvaluators with default column expressions.

        Args:
            future: Future contract code column expression
            bond: Bond code column expression
            date: Evaluation date column expression
            future_price: Future price column expression
            bond_ytm: Bond yield to maturity column expression
            capital_rate: Capital cost rate column expression
            reinvest_rate: Reinvestment rate (optional)
        """
        self.future = parse_into_expr(
            future if future is not None else pl.lit(None).cast(str)
        )
        self.bond = parse_into_expr(
            bond if bond is not None else pl.lit(None).cast(str)
        )
        self.date = parse_into_expr(
            date if date is not None else pl.lit(None).cast(pl.Date)
        )
        self.future_price = parse_into_expr(
            future_price if future_price is not None else pl.lit(None)
        )
        self.bond_ytm = parse_into_expr(
            bond_ytm if bond_ytm is not None else pl.lit(None)
        )
        self.capital_rate = parse_into_expr(
            capital_rate if capital_rate is not None else pl.lit(None)
        )
        self.reinvest_rate = reinvest_rate

    def _call_plugin(self, symbol: str):
        """Helper method to call plugin with consistent arguments."""
        return register_plugin(
            args=[
                self.future,
                self.bond,
                self.date,
                self.future_price,
                self.bond_ytm,
                self.capital_rate,
            ],
            kwargs={"reinvest_rate": self.reinvest_rate},
            symbol=symbol,
            is_elementwise=False,
        )

    @property
    def net_basis_spread(self):
        """
        Calculate net basis spread (净基差).

        Net basis spread = basis spread - carry return

        Returns:
            Polars expression for net basis spread
        """
        return self._call_plugin("evaluators_net_basis_spread")

    @property
    def accrued_interest(self):
        """
        Calculate accrued interest (应计利息).

        Returns:
            Polars expression for accrued interest
        """
        return self._call_plugin("evaluators_accrued_interest")

    @property
    def deliver_accrued_interest(self):
        """
        Calculate delivery accrued interest (国债期货交割应计利息).

        Returns:
            Polars expression for delivery accrued interest
        """
        return self._call_plugin("evaluators_deliver_accrued_interest")

    @property
    def cf(self):
        """
        Calculate conversion factor (转换因子).

        Returns:
            Polars expression for conversion factor
        """
        return self._call_plugin("evaluators_cf")

    @property
    def dirty_price(self):
        """
        Calculate bond dirty price (债券全价).

        Returns:
            Polars expression for bond dirty price
        """
        return self._call_plugin("evaluators_dirty_price")

    @property
    def clean_price(self):
        """
        Calculate bond clean price (债券净价).

        Returns:
            Polars expression for bond clean price
        """
        return self._call_plugin("evaluators_clean_price")

    @property
    def future_dirty_price(self):
        """
        Calculate future dirty price (期货全价/发票价格).

        Returns:
            Polars expression for future dirty price
        """
        return self._call_plugin("evaluators_future_dirty_price")

    @property
    def deliver_cost(self):
        """
        Calculate delivery cost (交割成本).

        Delivery cost = bond dirty price - interim coupon payments

        Returns:
            Polars expression for delivery cost
        """
        return self._call_plugin("evaluators_deliver_cost")

    @property
    def basis_spread(self):
        """
        Calculate basis spread (基差).

        Returns:
            Polars expression for basis spread
        """
        return self._call_plugin("evaluators_basis_spread")

    @property
    def f_b_spread(self):
        """
        Calculate futures-bond spread (期现价差).

        Returns:
            Polars expression for futures-bond spread
        """
        return self._call_plugin("evaluators_f_b_spread")

    @property
    def carry(self):
        """
        Calculate carry return (持有收益).

        Carry return = (delivery accrued - trading accrued + interim coupons) +
                      capital cost rate * (weighted average interim coupons - bond dirty price * remaining days / 365)

        Returns:
            Polars expression for carry return
        """
        return self._call_plugin("evaluators_carry")

    @property
    def duration(self):
        """
        Calculate modified duration (修正久期).

        Returns:
            Polars expression for modified duration
        """
        return self._call_plugin("evaluators_duration")

    @property
    def irr(self):
        """
        Calculate internal rate of return (内部收益率).

        Returns:
            Polars expression for internal rate of return
        """
        return self._call_plugin("evaluators_irr")

    @property
    def future_ytm(self):
        """
        Calculate futures implied yield to maturity (期货隐含收益率).

        Returns:
            Polars expression for futures implied yield to maturity
        """
        return self._call_plugin("evaluators_future_ytm")

    @property
    def remain_cp_to_deliver(self):
        """
        Calculate remaining coupon payments to delivery (到交割的期间付息).

        Returns:
            Polars expression for remaining coupon payments to delivery
        """
        return self._call_plugin("evaluators_remain_cp_to_deliver")

    @property
    def remain_cp_to_deliver_wm(self):
        """
        Calculate weighted average remaining coupon payments to delivery (加权平均到交割的期间付息).

        Returns:
            Polars expression for weighted average remaining coupon payments to delivery
        """
        return self._call_plugin("evaluators_remain_cp_to_deliver_wm")

    @property
    def remain_cp_num(self):
        """
        Calculate remaining number of coupon payments (债券剩余付息次数).

        Returns:
            Polars expression for remaining number of coupon payments
        """
        return self._call_plugin("evaluators_remain_cp_num")

    @property
    def deliver_date(self):
        """
        Calculate delivery date (交割日).

        Returns:
            Polars expression for delivery date
        """
        return self._call_plugin("evaluators_deliver_date")

    @property
    def last_trading_date(self):
        """
        Calculate last trading date (最后交易日).

        Returns:
            Polars expression for last trading date
        """
        return self._call_plugin("evaluators_last_trading_date")


class Bonds:
    """
    A class for bond-specific calculations using Polars expressions.

    This class provides methods for calculating various bond metrics
    without requiring futures contract information.
    """

    def __init__(self, bond: IntoExpr = "symbol"):
        """
        Initialize Bonds with bond identifier.

        Args:
            bond: Bond code column expression (default: "symbol")
        """
        self.bond = bond

    def _evaluator(
        self, date: IntoExpr | None = None, ytm: IntoExpr | None = None
    ) -> TfEvaluators:
        """
        Create a TfEvaluators instance for bond-only calculations.

        Args:
            date: Evaluation date column expression
            ytm: Yield to maturity column expression

        Returns:
            TfEvaluators: Configured evaluator instance
        """
        return TfEvaluators(
            future=None,
            bond=self.bond,
            date=date,
            bond_ytm=ytm,
            future_price=None,
            capital_rate=None,
            reinvest_rate=None,
        )

    def accrued_interest(self, date: IntoExpr = "date"):
        """
        Calculate accrued interest for the bond (应计利息).

        Args:
            date: Evaluation date column expression

        Returns:
            Polars expression for accrued interest
        """
        return self._evaluator(date=date).accrued_interest

    def clean_price(self, ytm: IntoExpr = "ytm", date: IntoExpr = "date"):
        """
        Calculate bond clean price (债券净价).

        Args:
            ytm: Yield to maturity column expression
            date: Evaluation date column expression

        Returns:
            Polars expression for bond clean price
        """
        return self._evaluator(date=date, ytm=ytm).clean_price

    def dirty_price(self, ytm: IntoExpr = "ytm", date: IntoExpr = "date"):
        """
        Calculate bond dirty price (债券全价).

        Args:
            ytm: Yield to maturity column expression
            date: Evaluation date column expression

        Returns:
            Polars expression for bond dirty price
        """
        return self._evaluator(date=date, ytm=ytm).dirty_price

    def duration(self, ytm: IntoExpr = "ytm", date: IntoExpr = "date"):
        """
        Calculate modified duration (修正久期).

        Args:
            ytm: Yield to maturity column expression
            date: Evaluation date column expression

        Returns:
            Polars expression for modified duration
        """
        return self._evaluator(date=date, ytm=ytm).duration

    def remain_cp_num(self, date: IntoExpr = "date"):
        """
        Calculate remaining number of coupon payments (债券剩余付息次数).

        Args:
            date: Evaluation date column expression

        Returns:
            Polars expression for remaining number of coupon payments
        """
        return self._evaluator(date=date).remain_cp_num

    # TODO(Teamon): 实现向量化根据净价反推ytm的函数


class Futures:
    def __init__(self, future: IntoExpr = "symbol"):
        """
        Initialize Futures with future identifier.

        Args:
            future: Future code column expression (default: "symbol")
        """
        self.future = future

    def _evaluator(self, date: IntoExpr | None = None) -> TfEvaluators:
        """
        Create a TfEvaluators instance for future-only calculations.

        Args:
            date: Evaluation date column expression

        Returns:
            TfEvaluators: Configured evaluator instance
        """
        return TfEvaluators(
            future=self.future,
            bond=None,
            date=date,
            bond_ytm=None,
            future_price=None,
            capital_rate=None,
            reinvest_rate=None,
        )

    def deliver_date(self):
        """
        Calculate delivery date (交割日).

        Args:
            date: Evaluation date column expression

        Returns:
            Polars expression for delivery date
        """
        return self._evaluator().deliver_date

    def last_trading_date(self):
        """
        Calculate last trading date (最后交易日).

        Args:
            date: Evaluation date column expression

        Returns:
            Polars expression for last trading date
        """
        return self._evaluator().last_trading_date


def find_workday(date: IntoExpr, market: str | Ib | Sse, offset: int = 0):
    """
    Find the workday based on the given date and market calendar.

    Args:
        date: Input date column expression
        market: Market identifier (IB, SSE, or string)
        offset: Number of workdays to offset (default: 0)

    Returns:
        Polars expression for the adjusted workday
    """
    if market == Ib:
        market = "IB"
    elif market == Sse:
        market = "SSE"
    date = parse_into_expr(date)
    return register_plugin(
        args=[date],
        kwargs={"market": market, "offset": offset},
        symbol="calendar_find_workday",
        is_elementwise=True,
    )


def is_business_day(date: IntoExpr, market: str | Ib | Sse):
    """
    Check if the given date is a business day for the specified market.

    Args:
        date: Input date column expression
        market: Market identifier (IB, SSE, or string)

    Returns:
        Polars expression returning boolean values for business day check
    """
    if market == Ib:
        market = "IB"
    elif market == Sse:
        market = "SSE"
    date = parse_into_expr(date)
    return register_plugin(
        args=[date],
        kwargs={"market": market},
        symbol="calendar_is_business_day",
        is_elementwise=True,
    )
