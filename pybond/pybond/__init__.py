from __future__ import annotations


from .pybond import TfEvaluator as _TfEvaluatorRS, Future
from .bond import Bond

from .nb import *


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


# __all__ = ["Bond", "Future", "TfEvaluator", "wind_login"]
