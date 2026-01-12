from __future__ import annotations

from .bond import Bond
from .pybond import Future, Ib, Sse, get_version, update_info_from_wind_sql_df
from .pybond import TfEvaluator as _TfEvaluatorRS

__version__ = get_version()


def update_info(df):
    import polars as pl

    if type(df).__module__.split(".")[0] == "pandas":
        df = pl.from_pandas(df)
    df.columns = [x.lower() for x in df.columns]
    schema = df.schema
    if schema.get("b_info_carrydate") != pl.String:
        df = df.with_columns(pl.col("b_info_carrydate").dt.strftime("%Y%m%d"))
    if schema.get("b_info_maturitydate") != pl.String:
        df = df.with_columns(pl.col("b_info_maturitydate").dt.strftime("%Y%m%d"))

    return update_info_from_wind_sql_df(df)


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            # 便于直接从Wind下载债券基础数据
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "Ib", "Sse", "TfEvaluator", "__version__"]
