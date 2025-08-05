from __future__ import annotations

from .bond import Bond
from .pybond import Future, Ib, Sse
from .pybond import TfEvaluator as _TfEvaluatorRS


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            # 便于直接从Wind下载债券基础数据
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "Ib", "Sse", "TfEvaluator"]
