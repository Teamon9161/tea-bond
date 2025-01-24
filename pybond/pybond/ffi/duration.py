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
