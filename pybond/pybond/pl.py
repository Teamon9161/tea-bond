from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from polars.type_aliases import IntoExpr

from .polars_utils import parse_into_expr, register_plugin

def calc_nbs(
    future: IntoExpr="future",
    bond: IntoExpr="bond",
    date: IntoExpr="date",
    future_price: IntoExpr="future_price",
    bond_ytm: IntoExpr="bond_ytm",
    capital_rate: IntoExpr="capital_rate",
    reinvest_rate=None,
):
    future = parse_into_expr(future)
    bond = parse_into_expr(bond)
    date = parse_into_expr(date)
    future_price = parse_into_expr(future_price)
    bond_ytm = parse_into_expr(bond_ytm)
    capital_rate = parse_into_expr(capital_rate)
    reinvest_rate = parse_into_expr(reinvest_rate)
    return register_plugin(
        args=[future, bond, date, future_price, bond_ytm, capital_rate],
        kwargs={"reinvest_rate": None},
        symbol="evaluators_net_basis_spread",
        is_elementwise=False,
    )
