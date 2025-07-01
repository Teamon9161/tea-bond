from __future__ import annotations

from .bond import Bond

# from .pnl import calc_bond_trade_pnl
from .tea_bond import Future, Ib, Sse
from .tea_bond import TfEvaluator as _TfEvaluatorRS


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "TfEvaluator", "Ib", "Sse"]
