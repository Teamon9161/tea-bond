from datetime import date, time

from numba import njit
from polars.datatypes.classes import TemporalType

from pybond.nb import Bond, TfEvaluator


@njit
def test_bond():
    # print(DateTime(dt))
    # time = Time(17, 28, 0)
    # print(time)
    _time = time(17, 28, 0)
    print(f"time; {_time}")
    bond = Bond("240018.IB")
    dt = date(2024, 12, 30)
    ytm = 0.019
    # print(bond)
    print(bond.full_code)
    print(bond.coupon_rate)
    print(bond.accrued_interest(dt))
    print(bond.dirty_price(ytm, dt))
    print(bond.clean_price(ytm, dt))
    print(bond.duration(ytm, dt))
    print(bond.calc_ytm_with_price(bond.dirty_price(ytm, dt), dt))

@njit
def test_evaluator():
    dt = date(2024, 12, 30)
    e = TfEvaluator("T2503", "240215.IB", dt, 100.0, 0.02, 0.018, 0.0).calc_all()
    # print(e.clean_price)
    # print(e.accrued_interest)
    # print(e.net_basis_spread)
    # print(e.irr)
    # print(e.carry)
    # print(e.future_ytm)
    print(e)

test_evaluator()

# py_e = TfEvaluator("T2503", "240215.IB", date(2024, 12, 30), 100.0, 0.02, 0.018, 0.0)
# print(py_e)
# print(TfEvaluator.from_ptr(py_e.ptr))
# print("===pyresult===")
# print(py_e.clean_price)
# print(py_e.accrued_interest)
# print(py_e.net_basis_spread)
# print(py_e.irr)
# print(py_e.carry)
# print(py_e.future_ytm)
