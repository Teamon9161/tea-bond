# from pybond import Bond, TfEvaluator

# bond = Bond.download("250002.IB", save=False)
# print(bond)
from numba import njit
from datetime import date, time
from pybond.nb import Bond, DateTime
from numba.types import string
# from hftbacktest import Backtester

@njit
def test():
    dt = DateTime(1249512)
    # dt = date(2024, 12, 10)
    # dt = time(10, 12, 21)
    # print(dt)
    # bond = Bond("240011")
    # dt = date(2024, 12, 16)
    # print(bond.duration(0.02, dt))
    # s = string("240018")
    # print(s._length, s._kind, s._is_ascii)

# Bond(240011).duration(0.02, "2024-12-16")
test()

# date = "2024-10-20"
# ytm = 1.89 / 100

# print(bond.accrued_interest(date))
# print(bond.dirty_price(ytm, date))
# print(bond.clean_price(ytm, date))
# print(bond.duration(ytm, date))
# print(bond.macaulay_duration(ytm, date))
