from __future__ import annotations

import pandas as pd
import polars as pl

from .pl import Bonds as PlBonds
from .pl import Futures as PlFutures
from .pl import TfEvaluators as PlTfEvaluators


class TfEvaluators:
    """
    Treasury Futures Evaluators for pandas DataFrames.

    A pandas-compatible wrapper around the Polars-based TfEvaluators that provides
    various financial calculations for treasury futures and bonds analysis.

    This class converts pandas inputs to Polars for computation and returns
    pandas Series results for seamless integration with pandas workflows.
    """

    def __init__(
        self,
        future: str | pd.Series,
        bond: str | pd.Series,
        date: str | pd.Series,
        future_price: pd.Series | None = None,
        bond_ytm: pd.Series | None = None,
        capital_rate: float | pd.Series | None = None,
        reinvest_rate: float | None = None,
    ):
        """
        Initialize TfEvaluators with market data.

        Args:
            future: Future contract code(s)
            bond: Bond code(s)
            date: Evaluation date(s)
            future_price: Future price(s)
            bond_ytm: Bond yield to maturity
            capital_rate: Capital cost rate for carry calculations
            reinvest_rate: Reinvestment rate for coupon payments (optional)
        """
        self.pl_df = pl.DataFrame(
            {
                "future": future,
                "bond": bond,
                "date": date,
                "future_price": future_price,
                "bond_ytm": bond_ytm,
                "capital_rate": capital_rate,
            }
        )
        self._evaluators = PlTfEvaluators(
            future_price="future_price",
            bond_ytm="bond_ytm",
            capital_rate="capital_rate",
            reinvest_rate=reinvest_rate,
        )

    @property
    def net_basis_spread(self):
        """
        Calculate net basis spread (净基差).

        Net basis spread = basis spread - carry return

        Returns:
            pd.Series: Net basis spread values
        """
        return self.pl_df.select(net_basis_spread=self._evaluators.net_basis_spread)[
            "net_basis_spread"
        ].to_pandas()

    @property
    def accrued_interest(self):
        """
        Calculate accrued interest (应计利息).

        Returns:
            pd.Series: Accrued interest values
        """
        return self.pl_df.select(accrued_interest=self._evaluators.accrued_interest)[
            "accrued_interest"
        ].to_pandas()

    @property
    def deliver_accrued_interest(self):
        """
        Calculate delivery accrued interest (国债期货交割应计利息).

        Returns:
            pd.Series: Delivery accrued interest values
        """
        return self.pl_df.select(
            deliver_accrued_interest=self._evaluators.deliver_accrued_interest
        )["deliver_accrued_interest"].to_pandas()

    @property
    def cf(self):
        """
        Calculate conversion factor (转换因子).

        Returns:
            pd.Series: Conversion factor values
        """
        return self.pl_df.select(cf=self._evaluators.cf)["cf"].to_pandas()

    @property
    def dirty_price(self):
        """
        Calculate bond dirty price (债券全价).

        Returns:
            pd.Series: Bond dirty price values
        """
        return self.pl_df.select(dirty_price=self._evaluators.dirty_price)[
            "dirty_price"
        ].to_pandas()

    @property
    def clean_price(self):
        """
        Calculate bond clean price (债券净价).

        Returns:
            pd.Series: Bond clean price values
        """
        return self.pl_df.select(clean_price=self._evaluators.clean_price)[
            "clean_price"
        ].to_pandas()

    @property
    def future_dirty_price(self):
        """
        Calculate future dirty price (期货全价/发票价格).

        Returns:
            pd.Series: Future dirty price values
        """
        return self.pl_df.select(
            future_dirty_price=self._evaluators.future_dirty_price
        )["future_dirty_price"].to_pandas()

    @property
    def deliver_cost(self):
        """
        Calculate delivery cost (交割成本).

        Delivery cost = bond dirty price - interim coupon payments

        Returns:
            pd.Series: Delivery cost values
        """
        return self.pl_df.select(deliver_cost=self._evaluators.deliver_cost)[
            "deliver_cost"
        ].to_pandas()

    @property
    def basis_spread(self):
        """
        Calculate basis spread (基差).

        Returns:
            pd.Series: Basis spread values
        """
        return self.pl_df.select(basis_spread=self._evaluators.basis_spread)[
            "basis_spread"
        ].to_pandas()

    @property
    def f_b_spread(self):
        """
        Calculate futures-bond spread (期现价差).

        Returns:
            pd.Series: Futures-bond spread values
        """
        return self.pl_df.select(f_b_spread=self._evaluators.f_b_spread)[
            "f_b_spread"
        ].to_pandas()

    @property
    def carry(self):
        """
        Calculate carry return (持有收益).

        Carry return = (delivery accrued - trading accrued + interim coupons) +
                      capital cost rate * (weighted average interim coupons - bond dirty price * remaining days / 365)

        Returns:
            pd.Series: Carry return values
        """
        return self.pl_df.select(carry=self._evaluators.carry)["carry"].to_pandas()

    @property
    def duration(self):
        """
        Calculate modified duration (修正久期).

        Returns:
            pd.Series: Modified duration values
        """
        return self.pl_df.select(duration=self._evaluators.duration)[
            "duration"
        ].to_pandas()

    @property
    def irr(self):
        """
        Calculate internal rate of return (内部收益率).

        Returns:
            pd.Series: Internal rate of return values
        """
        return self.pl_df.select(irr=self._evaluators.irr)["irr"].to_pandas()

    @property
    def future_ytm(self):
        """
        Calculate futures implied yield to maturity (期货隐含收益率).

        Returns:
            pd.Series: Futures implied yield to maturity values
        """
        return self.pl_df.select(future_ytm=self._evaluators.future_ytm)[
            "future_ytm"
        ].to_pandas()

    @property
    def remain_cp_to_deliver(self):
        """
        Calculate remaining coupon payments to delivery (到交割的期间付息).

        Returns:
            pd.Series: Remaining coupon payments to delivery values
        """
        return self.pl_df.select(
            remain_cp_to_deliver=self._evaluators.remain_cp_to_deliver
        )["remain_cp_to_deliver"].to_pandas()

    @property
    def remain_cp_to_deliver_wm(self):
        """
        Calculate weighted average remaining coupon payments to delivery (加权平均到交割的期间付息).

        Returns:
            pd.Series: Weighted average remaining coupon payments to delivery values
        """
        return self.pl_df.select(
            remain_cp_to_deliver_wm=self._evaluators.remain_cp_to_deliver_wm
        )["remain_cp_to_deliver_wm"].to_pandas()

    @property
    def remain_cp_num(self):
        """
        Calculate remaining number of coupon payments (债券剩余付息次数).

        Returns:
            pd.Series: Remaining number of coupon payments values
        """
        return self.pl_df.select(remain_cp_num=self._evaluators.remain_cp_num)[
            "remain_cp_num"
        ].to_pandas()

    @property
    def deliver_date(self):
        """
        Calculate delivery date (交割日).

        Returns:
            pd.Series: Delivery date values
        """
        return self.pl_df.select(deliver_date=self._evaluators.deliver_date)[
            "deliver_date"
        ].to_pandas()

    @property
    def last_trading_date(self):
        """
        Calculate last trading date (最后交易日).

        Returns:
            pd.Series: Last trading date values
        """
        return self.pl_df.select(last_trading_date=self._evaluators.last_trading_date)[
            "last_trading_date"
        ].to_pandas()


class Bonds:
    """
    Bond calculations for pandas DataFrames.

    A pandas-compatible wrapper around the Polars-based Bonds class that provides
    bond-specific financial calculations without requiring futures contract information.
    """

    def __init__(self, bond: str | pd.Series):
        """
        Initialize Bonds with bond identifier.

        Args:
            bond: Bond code(s)
        """
        self.bond = bond

    def accrued_interest(self, date: str | pd.Series):
        """
        Calculate accrued interest for the bond (应计利息).

        Args:
            date: Evaluation date(s)

        Returns:
            pd.Series: Accrued interest values
        """
        df = pl.DataFrame({"bond": self.bond, "date": date})
        return df.select(accrued_interest=PlBonds("bond").accrued_interest("date"))[
            "accrued_interest"
        ].to_pandas()

    def clean_price(self, ytm: float | pd.Series, date: str | pd.Series):
        """
        Calculate bond clean price (债券净价).

        Args:
            ytm: Yield to maturity
            date: Evaluation date(s)

        Returns:
            pd.Series: Bond clean price values
        """
        df = pl.DataFrame({"bond": self.bond, "ytm": ytm, "date": date})
        return df.select(clean_price=PlBonds("bond").clean_price("ytm", "date"))[
            "clean_price"
        ].to_pandas()

    def dirty_price(self, ytm: float | pd.Series, date: str | pd.Series):
        """
        Calculate bond dirty price (债券全价).

        Args:
            ytm: Yield to maturity
            date: Evaluation date(s)

        Returns:
            pd.Series: Bond dirty price values
        """
        df = pl.DataFrame({"bond": self.bond, "ytm": ytm, "date": date})
        return df.select(dirty_price=PlBonds("bond").dirty_price("ytm", "date"))[
            "dirty_price"
        ].to_pandas()

    def duration(self, ytm: float | pd.Series, date: str | pd.Series):
        """
        Calculate modified duration (修正久期).

        Args:
            ytm: Yield to maturity
            date: Evaluation date(s)

        Returns:
            pd.Series: Modified duration values
        """
        df = pl.DataFrame({"bond": self.bond, "ytm": ytm, "date": date})
        return df.select(duration=PlBonds("bond").duration("ytm", "date"))[
            "duration"
        ].to_pandas()

    def remain_cp_num(self, date: str | pd.Series):
        """
        Calculate remaining number of coupon payments (债券剩余付息次数).

        Args:
            date: Evaluation date(s)

        Returns:
            pd.Series: Remaining number of coupon payments values
        """
        df = pl.DataFrame({"bond": self.bond, "date": date})
        return df.select(remain_cp_num=PlBonds("bond").remain_cp_num("date"))[
            "remain_cp_num"
        ].to_pandas()


class Futures:
    def __init__(self, future: str | pd.Series):
        self.future = future

    def deliver_date(self):
        """
        Calculate delivery date (交割日).

        Args:
            date: Evaluation date(s)

        Returns:
            pd.Series: Delivery date values
        """
        df = pl.DataFrame({"future": self.future})
        return df.select(deliver_date=PlFutures("future").deliver_date())[
            "deliver_date"
        ].to_pandas()

    def last_trading_date(self):
        """
        Calculate last trading date (最后交易日).

        Args:
            date: Evaluation date(s)

        Returns:
            pd.Series: Last trading date values
        """
        df = pl.DataFrame({"future": self.future})
        return df.select(last_trading_date=PlFutures("future").last_trading_date())[
            "last_trading_date"
        ].to_pandas()


def find_workday(date: str | pd.Series, market: str, offset: int = 0):
    """
    Find the workday based on the given date and market calendar.

    Args:
        date: Input date(s)
        market: Market identifier ("IB" or "SSE")
        offset: Number of workdays to offset (default: 0)

    Returns:
        pd.Series: Adjusted workday values
    """
    from .pl import find_workday as pl_find_workday

    df = pl.DataFrame({"date": date}).select(pl.col("date").dt.date())
    return df.select(workday=pl_find_workday("date", market, offset))[
        "workday"
    ].to_pandas()


def is_business_day(date: str | pd.Series, market: str):
    """
    Check if the given date is a business day for the specified market.

    Args:
        date: Input date(s)
        market: Market identifier ("IB" or "SSE")

    Returns:
        pd.Series: Boolean values indicating if dates are business days
    """
    from .pl import is_business_day as pl_is_business_day

    df = pl.DataFrame({"date": date}).select(pl.col("date").dt.date())
    return df.select(is_business=pl_is_business_day("date", market))[
        "is_business"
    ].to_pandas()
