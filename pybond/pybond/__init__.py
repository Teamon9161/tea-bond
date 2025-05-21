from __future__ import annotations

from .bond import Bond
from .pybond import Future
from .pybond import TfEvaluator as _TfEvaluatorRS
from .pybond import IB, SSE

class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "TfEvaluator", "IB", "SSE"]
