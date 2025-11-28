from __future__ import annotations

from .bond import Bond
from .pybond import Future, Ib, Sse, get_version, update_info_from_wind_sql_df
from .pybond import TfEvaluator as _TfEvaluatorRS

__version__ = get_version()


def update_info(df):
    if type(df).__module__.split(".")[0] == "pandas":
        import polars as pl

        df = pl.from_pandas(df)
    return update_info_from_wind_sql_df(df)


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            # 便于直接从Wind下载债券基础数据
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "Ib", "Sse", "TfEvaluator", "__version__"]
