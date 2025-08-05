import datetime

import numpy as np
import polars as pl

# import os
# os.environ["POLARS_VERBOSE"] = "1"
from pybond import Ib, TfEvaluator
from pybond.pd import Bonds as PdBonds
from pybond.pd import TfEvaluators as PdTfEvaluators
from pybond.pd import find_workday as pd_find_workday
from pybond.pl import Bonds, TfEvaluators, find_workday, is_business_day

e = TfEvaluator("T2509", 250205, "2025-07-15", 100, 0.02, 0.018)
e.net_basis_spread


length = 1000000
futures = ["T2509"] * (length - 2) + ["T2412", "T2509"]
bonds = ["240215"] * (length - 1) + ["240018"]
dates = [datetime.date(2025, 5, 11)] * (length - 3) + [datetime.date(2025, 5, 15)] * 3
future_prices = np.random.rand(length) + 102
bond_ytms = np.random.rand(length) * 0.001 + 0.02


df = pl.DataFrame(
    {
        "future": futures,
        "bond": bonds,
        "date": dates,
        "future_price": future_prices,
        "bond_ytm": bond_ytms,
    }
)

pd_df = df.to_pandas()

print(PdBonds(pd_df["bond"]).clean_price(0.019, pd_df["date"]))

print(
    PdTfEvaluators(
        "T2509",
        pd_df["bond"],
        df["date"],
        pd_df["future_price"],
        pd_df["bond_ytm"],
        capital_rate=0.016,
    ).net_basis_spread
)

print(pd_find_workday(pd_df["date"], Ib, 0))

print(
    df.select(
        "date",
        sd=find_workday("date", "IB", 1),
        ib=is_business_day("date", "SSE"),
        cp=Bonds("bond").clean_price(ytm="bond_ytm"),
        dp=Bonds("bond").dirty_price(ytm="bond_ytm"),
        ai=Bonds("bond").accrued_interest(),
        dv=Bonds("bond").duration(ytm="bond_ytm"),
    )
)

# import time

# start = time.perf_counter()
# print(df.select(TfEvaluators(capital_rate=0.018).net_basis_spread.alias("nbs")))
# # res = []
# # evaluator = TfEvaluator(
# #     "T2412", "", datetime.date(1970, 1, 1), np.nan, np.nan, 0.018, 0
# # )
# # for i in range(length):
# #     # evaluator = TfEvaluator(futures[i], bonds[i], dates[i], future_prices[i], bond_ytms[i], 0.018)
# #     evaluator = evaluator.update(
# #         future_prices[i], bond_ytms[i], dates[i], futures[i], bonds[i], 0.018
# #     )
# #     res.append(evaluator.net_basis_spread)
# # print(np.array(res))
# print(f"Time taken: {time.perf_counter() - start:.6f} seconds")
