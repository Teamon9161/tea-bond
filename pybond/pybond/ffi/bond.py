import ctypes

from .lib import lib

create_bond = lib.create_bond
create_bond.argtypes = (ctypes.c_void_p, ctypes.c_size_t)
create_bond.restype = ctypes.c_void_p

free_bond = lib.free_bond
free_bond.argtypes = [ctypes.c_void_p]
free_bond.restype = None

bond_coupon_rate = lib.bond_coupon_rate
bond_coupon_rate.argtypes = [ctypes.c_void_p]
bond_coupon_rate.restype = ctypes.c_double

bond_full_code = lib.bond_full_code
bond_full_code.argtypes = [ctypes.c_void_p]
bond_full_code.restype = ctypes.c_char_p

bond_calc_ytm = lib.bond_calc_ytm
bond_calc_ytm.argtypes = [
    ctypes.c_void_p,
    ctypes.c_double,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
]
bond_calc_ytm.restype = ctypes.c_double

bond_duration = lib.bond_duration
bond_duration.argtypes = [
    ctypes.c_void_p,
    ctypes.c_double,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
]
bond_duration.restype = ctypes.c_double

bond_accrued_interest = lib.bond_accrued_interest
bond_accrued_interest.argtypes = [
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
]
bond_accrued_interest.restype = ctypes.c_double

bond_dirty_price = lib.bond_dirty_price
bond_dirty_price.argtypes = [
    ctypes.c_void_p,
    ctypes.c_double,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
]
bond_dirty_price.restype = ctypes.c_double

bond_clean_price = lib.bond_clean_price
bond_clean_price.argtypes = [
    ctypes.c_void_p,
    ctypes.c_double,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
]
bond_clean_price.restype = ctypes.c_double
