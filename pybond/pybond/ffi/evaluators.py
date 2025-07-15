import ctypes

from .lib import lib

# TfEvaluator creation and destruction
create_tf_evaluator = lib.create_tf_evaluator
create_tf_evaluator.argtypes = [
    ctypes.c_void_p,  # future_code_ptr
    ctypes.c_size_t,  # future_code_len
    ctypes.c_double,  # future_price
    ctypes.c_void_p,  # bond_code_ptr
    ctypes.c_size_t,  # bond_code_len
    ctypes.c_double,  # bond_ytm
    ctypes.c_double,  # capital_rate
    ctypes.c_uint32,  # year
    ctypes.c_uint32,  # month
    ctypes.c_uint32,  # day
]
create_tf_evaluator.restype = ctypes.c_void_p

create_tf_evaluator_with_reinvest = lib.create_tf_evaluator_with_reinvest
create_tf_evaluator_with_reinvest.argtypes = [
    ctypes.c_void_p,  # future_code_ptr
    ctypes.c_size_t,  # future_code_len
    ctypes.c_double,  # future_price
    ctypes.c_void_p,  # bond_code_ptr
    ctypes.c_size_t,  # bond_code_len
    ctypes.c_double,  # bond_ytm
    ctypes.c_double,  # capital_rate
    ctypes.c_double,  # reinvest_rate
    ctypes.c_uint32,  # year
    ctypes.c_uint32,  # month
    ctypes.c_uint32,  # day
]
create_tf_evaluator_with_reinvest.restype = ctypes.c_void_p

free_tf_evaluator = lib.free_tf_evaluator
free_tf_evaluator.argtypes = [ctypes.c_void_p]
free_tf_evaluator.restype = None

# Basic properties
tf_evaluator_is_deliverable = lib.tf_evaluator_is_deliverable
tf_evaluator_is_deliverable.argtypes = [ctypes.c_void_p]
tf_evaluator_is_deliverable.restype = ctypes.c_int

tf_evaluator_bond_code = lib.tf_evaluator_bond_code
tf_evaluator_bond_code.argtypes = [ctypes.c_void_p]
tf_evaluator_bond_code.restype = ctypes.c_char_p

tf_evaluator_future_code = lib.tf_evaluator_future_code
tf_evaluator_future_code.argtypes = [ctypes.c_void_p]
tf_evaluator_future_code.restype = ctypes.c_char_p

tf_evaluator_bond_ytm = lib.tf_evaluator_bond_ytm
tf_evaluator_bond_ytm.argtypes = [ctypes.c_void_p]
tf_evaluator_bond_ytm.restype = ctypes.c_double

tf_evaluator_future_price = lib.tf_evaluator_future_price
tf_evaluator_future_price.argtypes = [ctypes.c_void_p]
tf_evaluator_future_price.restype = ctypes.c_double

tf_evaluator_capital_rate = lib.tf_evaluator_capital_rate
tf_evaluator_capital_rate.argtypes = [ctypes.c_void_p]
tf_evaluator_capital_rate.restype = ctypes.c_double

tf_evaluator_reinvest_rate = lib.tf_evaluator_reinvest_rate
tf_evaluator_reinvest_rate.argtypes = [ctypes.c_void_p]
tf_evaluator_reinvest_rate.restype = ctypes.c_double

# Date functions
tf_evaluator_get_date = lib.tf_evaluator_get_date
tf_evaluator_get_date.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(ctypes.c_uint32),
]
tf_evaluator_get_date.restype = None

tf_evaluator_get_deliver_date = lib.tf_evaluator_get_deliver_date
tf_evaluator_get_deliver_date.argtypes = [
    ctypes.c_void_p,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.POINTER(ctypes.c_uint32),
]
tf_evaluator_get_deliver_date.restype = ctypes.c_int

# Calculation functions
tf_evaluator_accrued_interest = lib.tf_evaluator_accrued_interest
tf_evaluator_accrued_interest.argtypes = [ctypes.c_void_p]
tf_evaluator_accrued_interest.restype = ctypes.c_double

tf_evaluator_deliver_accrued_interest = lib.tf_evaluator_deliver_accrued_interest
tf_evaluator_deliver_accrued_interest.argtypes = [ctypes.c_void_p]
tf_evaluator_deliver_accrued_interest.restype = ctypes.c_double

tf_evaluator_cf = lib.tf_evaluator_cf
tf_evaluator_cf.argtypes = [ctypes.c_void_p]
tf_evaluator_cf.restype = ctypes.c_double

tf_evaluator_dirty_price = lib.tf_evaluator_dirty_price
tf_evaluator_dirty_price.argtypes = [ctypes.c_void_p]
tf_evaluator_dirty_price.restype = ctypes.c_double

tf_evaluator_clean_price = lib.tf_evaluator_clean_price
tf_evaluator_clean_price.argtypes = [ctypes.c_void_p]
tf_evaluator_clean_price.restype = ctypes.c_double

tf_evaluator_future_dirty_price = lib.tf_evaluator_future_dirty_price
tf_evaluator_future_dirty_price.argtypes = [ctypes.c_void_p]
tf_evaluator_future_dirty_price.restype = ctypes.c_double

tf_evaluator_deliver_cost = lib.tf_evaluator_deliver_cost
tf_evaluator_deliver_cost.argtypes = [ctypes.c_void_p]
tf_evaluator_deliver_cost.restype = ctypes.c_double

tf_evaluator_basis_spread = lib.tf_evaluator_basis_spread
tf_evaluator_basis_spread.argtypes = [ctypes.c_void_p]
tf_evaluator_basis_spread.restype = ctypes.c_double

tf_evaluator_f_b_spread = lib.tf_evaluator_f_b_spread
tf_evaluator_f_b_spread.argtypes = [ctypes.c_void_p]
tf_evaluator_f_b_spread.restype = ctypes.c_double

tf_evaluator_carry = lib.tf_evaluator_carry
tf_evaluator_carry.argtypes = [ctypes.c_void_p]
tf_evaluator_carry.restype = ctypes.c_double

tf_evaluator_net_basis_spread = lib.tf_evaluator_net_basis_spread
tf_evaluator_net_basis_spread.argtypes = [ctypes.c_void_p]
tf_evaluator_net_basis_spread.restype = ctypes.c_double

tf_evaluator_duration = lib.tf_evaluator_duration
tf_evaluator_duration.argtypes = [ctypes.c_void_p]
tf_evaluator_duration.restype = ctypes.c_double

tf_evaluator_irr = lib.tf_evaluator_irr
tf_evaluator_irr.argtypes = [ctypes.c_void_p]
tf_evaluator_irr.restype = ctypes.c_double

tf_evaluator_future_ytm = lib.tf_evaluator_future_ytm
tf_evaluator_future_ytm.argtypes = [ctypes.c_void_p]
tf_evaluator_future_ytm.restype = ctypes.c_double

tf_evaluator_remain_days_to_deliver = lib.tf_evaluator_remain_days_to_deliver
tf_evaluator_remain_days_to_deliver.argtypes = [ctypes.c_void_p]
tf_evaluator_remain_days_to_deliver.restype = ctypes.c_int

tf_evaluator_remain_cp_num = lib.tf_evaluator_remain_cp_num
tf_evaluator_remain_cp_num.argtypes = [ctypes.c_void_p]
tf_evaluator_remain_cp_num.restype = ctypes.c_int

tf_evaluator_remain_cp_to_deliver = lib.tf_evaluator_remain_cp_to_deliver
tf_evaluator_remain_cp_to_deliver.argtypes = [ctypes.c_void_p]
tf_evaluator_remain_cp_to_deliver.restype = ctypes.c_double

tf_evaluator_remain_cp_to_deliver_wm = lib.tf_evaluator_remain_cp_to_deliver_wm
tf_evaluator_remain_cp_to_deliver_wm.argtypes = [ctypes.c_void_p]
tf_evaluator_remain_cp_to_deliver_wm.restype = ctypes.c_double

tf_evaluator_calc_all = lib.tf_evaluator_calc_all
tf_evaluator_calc_all.argtypes = [ctypes.c_void_p]
tf_evaluator_calc_all.restype = ctypes.c_int

# Update function
tf_evaluator_update_info = lib.tf_evaluator_update_info
tf_evaluator_update_info.argtypes = [
    ctypes.c_void_p,  # evaluator
    ctypes.c_void_p,  # future_code_ptr
    ctypes.c_size_t,  # future_code_len
    ctypes.c_double,  # future_price
    ctypes.c_void_p,  # bond_code_ptr
    ctypes.c_size_t,  # bond_code_len
    ctypes.c_double,  # bond_ytm
    ctypes.c_double,  # capital_rate
    ctypes.c_uint32,  # year
    ctypes.c_uint32,  # month
    ctypes.c_uint32,  # day
]
tf_evaluator_update_info.restype = ctypes.c_int

# Utility function
free_string = lib.free_string
free_string.argtypes = [ctypes.c_char_p]
free_string.restype = None
