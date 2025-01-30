from numba import njit

from pybond.nb import *

# #[no_mangle]
# pub extern "C" fn build_datetime_ns(val: i64) -> *mut c_void {
#     let dt = Utc.timestamp_nanos(val).naive_utc();
#     Box::into_raw(Box::new(dt)) as *mut c_void
# }
#
# dt = datetime.now().timestamp() * 1e9
# dt


@njit
def test():
    # print(DateTime(dt))
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
    # t = Test(1249512)
    # t = DateTime(12401204)


test()
