from pybond.nb import TfEvaluator
import numba as nb
import numpy as np
import datetime


@nb.njit
def nbs_impl(futures, bonds, dates, future_prices, bond_ytms, capital_rate, reinvest_rate):
    length = len(bond_ytms)
    out = np.zeros(length)
    evaluator = TfEvaluator("T2412", "", datetime.date(1970, 1, 1), np.nan, np.nan, capital_rate, reinvest_rate)
    for i in range(length):
        evaluator.update(future_prices[i], bond_ytms[i], dates[i], futures[i], bonds[i], capital_rate)
        out[i] = evaluator.net_basis_spread
    return out

class TfEvaluators:
    def __init__(self, future, bond, date, future_price, bond_ytm, capital_rate=0.018, reinvest_rate=0.0):
        self.future = future
        self.bond = bond
        self.date = date
        self.future_price = future_price
        self.bond_ytm = bond_ytm
        self.capital_rate = capital_rate
        self.reinvest_rate = reinvest_rate

    def net_basis_spread(self):
        return nbs_impl(self.future, self.bond, self.date, self.future_price, self.bond_ytm, self.capital_rate, self.reinvest_rate)


# length = 1000000
# futures = ["T2509"] * length
# bonds = ["240215"] * length
# dates = [datetime.date(2025, 5, 11)] * length
# future_prices = np.random.rand(length) + 102
# bond_ytms = np.random.rand(length) * 0.001 + 0.02

# import time
# start = time.perf_counter()
# TfEvaluators(futures, bonds, dates, future_prices, bond_ytms).net_basis_spread()
# # res = []
# # evaluator = TfEvaluator("T2412", "", datetime.date(1970, 1, 1), np.nan, np.nan, 0.018, 0)
# # for i in range(length):
# #     evaluator = evaluator.update(future_prices[i], bond_ytms[i], dates[i], futures[i], bonds[i], 0.018)
# #     res.append(evaluator.net_basis_spread)

# print(f"Time taken: {time.perf_counter() - start:.6f} seconds")
