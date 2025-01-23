import ctypes

from .lib import lib

build_datetime_ns = lib.build_datetime_ns
build_datetime_ns.argtypes = (ctypes.c_int64,)
build_datetime_ns.restype = ctypes.c_void_p

build_datetime_from_utc_ns = lib.build_datetime_from_utc_ns
build_datetime_from_utc_ns.argtypes = (ctypes.c_int64,)
build_datetime_from_utc_ns.restype = ctypes.c_void_p

local_timestamp_nanos = lib.local_timestamp_nanos
local_timestamp_nanos.argtypes = (ctypes.c_void_p,)
local_timestamp_nanos.restype = ctypes.c_int64

timestamp_nanos = lib.timestamp_nanos
timestamp_nanos.argtypes = (ctypes.c_void_p,)
timestamp_nanos.restype = ctypes.c_int64

utc_timestamp_to_local = lib.utc_timestamp_to_local
utc_timestamp_to_local.argtypes = (ctypes.c_int64,)
utc_timestamp_to_local.restype = ctypes.c_int64

_free_datetime = lib.free_datetime
_free_datetime.argtypes = (ctypes.c_void_p,)

get_datetime_year = lib.get_datetime_year
get_datetime_year.argtypes = (ctypes.c_void_p,)
get_datetime_year.restype = ctypes.c_int32

get_datetime_month = lib.get_datetime_month
get_datetime_month.argtypes = (ctypes.c_void_p,)
get_datetime_month.restype = ctypes.c_int32

get_datetime_day = lib.get_datetime_day
get_datetime_day.argtypes = (ctypes.c_void_p,)
get_datetime_day.restype = ctypes.c_int32

get_datetime_hour = lib.get_datetime_hour
get_datetime_hour.argtypes = (ctypes.c_void_p,)
get_datetime_hour.restype = ctypes.c_int32

get_datetime_minute = lib.get_datetime_minute
get_datetime_minute.argtypes = (ctypes.c_void_p,)
get_datetime_minute.restype = ctypes.c_int32

get_datetime_second = lib.get_datetime_second
get_datetime_second.argtypes = (ctypes.c_void_p,)
get_datetime_second.restype = ctypes.c_int32

get_datetime_nanosecond = lib.get_datetime_nanosecond
get_datetime_nanosecond.argtypes = (ctypes.c_void_p,)
get_datetime_nanosecond.restype = ctypes.c_int32

datetime_with_time = lib.datetime_with_time
datetime_with_time.argtypes = (ctypes.c_void_p, (ctypes.c_uint32 * 6))
datetime_with_time.restype = ctypes.c_void_p
