# from pybond import Bond, TfEvaluator

# bond = Bond.download("250002.IB", save=False)
# print(bond)
from numba import njit

from pybond.nb import Bond


@njit
def test():
    bond = Bond("240018")
    print(bond.duration(0.02, 2024, 12, 16))


test()

# date = "2024-10-20"
# ytm = 1.89 / 100

# print(bond.accrued_interest(date))
# print(bond.dirty_price(ytm, date))
# print(bond.clean_price(ytm, date))
# print(bond.duration(ytm, date))
# print(bond.macaulay_duration(ytm, date))
