from __future__ import annotations

from typing import TYPE_CHECKING

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
        future_price: IntoExpr = "future_price",
        bond_ytm: IntoExpr = "bond_ytm",
        capital_rate: IntoExpr = "capital_rate",
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
        if future is None:
            future = pl.lit(None).cast(str)
        self.future = parse_into_expr(future)
        self.bond = parse_into_expr(bond)
        self.date = parse_into_expr(date)
        self.future_price = parse_into_expr(future_price)
        self.bond_ytm = parse_into_expr(bond_ytm)
        self.capital_rate = parse_into_expr(capital_rate)
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


class Bonds:
    def __init__(self, bond: IntoExpr = "symbol"):
        self.bond = bond

    def _evaluator(self, date=None, ytm=None) -> TfEvaluators:
        return TfEvaluators(
            future=None,
            bond=self.bond,
            date=date,
            bond_ytm=ytm,
            future_price=None,
            capital_rate=None,
            reinvest_rate=None,
        )

    def accrued_interest(self, date="date"):
        return self._evaluator(date=date).accrued_interest

    def clean_price(self, ytm="ytm", date="date"):
        return self._evaluator(date=date, ytm=ytm).clean_price

    def dirty_price(self, ytm="ytm", date="date"):
        return self._evaluator(date=date, ytm=ytm).dirty_price

    def duration(self, ytm="ytm", date="date"):
        return self._evaluator(date=date, ytm=ytm).duration

    def remain_cp_num(self, date="date"):
        return self._evaluator(date=date).remain_cp_num

    # TODO(Teamon): 实现向量化根据净价反推ytm的函数
