import time
from datetime import date, datetime

from numba import njit
from numba.types import string

from pybond.nb import Bond, DateTime, Duration

dt = datetime.now().timestamp() * 1e9


@njit
def test(dt):
    dt = DateTime(dt)
    print(dt)
    dr = Duration("1mo3h1m")
    dt2 = dt - dr
    print(dt2 > dt)
    # dt = DateTime(1249512)
    # dt = date(2024, 12, 10)
    # print(dt)
    # bond = Bond("240011.IB")
    # dt = date(2024, 12, 16)
    # print(bond.duration(0.02, dt))
    # s = string("240018")
    # print(s._length, s._kind, s._is_ascii)


# Bond(240011).duration(0.02, "2024-12-16")
test(dt)

# date = "2024-10-20"
# ytm = 1.89 / 100

# print(bond.accrued_interest(date))
# print(bond.dirty_price(ytm, date))
# print(bond.clean_price(ytm, date))
# print(bond.duration(ytm, date))
# print(bond.macaulay_duration(ytm, date))
