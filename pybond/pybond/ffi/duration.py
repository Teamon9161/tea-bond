import ctypes

from .lib import lib

parse_duration = lib.parse_duration
parse_duration.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
parse_duration.restype = ctypes.c_void_p

datetime_sub_datetime = lib.datetime_sub_datetime
datetime_sub_datetime.argtypes = [ctypes.c_int64, ctypes.c_int64]
datetime_sub_datetime.restype = ctypes.c_void_p

datetime_add_duration = lib.datetime_add_duration
datetime_add_duration.argtypes = [ctypes.c_int64, ctypes.c_void_p]
datetime_add_duration.restype = ctypes.c_int64

datetime_sub_duration = lib.datetime_sub_duration
datetime_sub_duration.argtypes = [ctypes.c_int64, ctypes.c_void_p]
datetime_sub_duration.restype = ctypes.c_int64

duration_ge_duration = lib.duration_ge_duration
duration_ge_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_ge_duration.restype = ctypes.c_bool

duration_gt_duration = lib.duration_gt_duration
duration_gt_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_gt_duration.restype = ctypes.c_bool

duration_eq_duration = lib.duration_eq_duration
duration_eq_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_eq_duration.restype = ctypes.c_bool

duration_ne_duration = lib.duration_ne_duration
duration_ne_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_ne_duration.restype = ctypes.c_bool

duration_lt_duration = lib.duration_lt_duration
duration_lt_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_lt_duration.restype = ctypes.c_bool

duration_le_duration = lib.duration_le_duration
duration_le_duration.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
duration_le_duration.restype = ctypes.c_bool
