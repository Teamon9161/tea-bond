import ctypes

from .lib import lib

create_bond = lib.create_bond
# create_bond.argstypes = (ctypes.c_void_p,)
create_bond.argstypes = (ctypes.c_void_p, ctypes.c_size_t)
create_bond.restype = ctypes.c_void_p

bond_duration = lib.bond_duration
bond_duration.argtypes = (
    ctypes.c_void_p,
    ctypes.c_double,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.c_uint32,
)
bond_duration.restype = ctypes.c_double
