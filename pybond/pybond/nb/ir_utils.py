from llvmlite import ir
from numba.core import cgutils


def ir_isinstance(pyapi, obj, typ):
    fnty = ir.FunctionType(ir.IntType(1), [pyapi.pyobj, pyapi.pyobj])
    fn = cgutils.get_or_insert_function(
        pyapi.builder.module, fnty, name="PyObject_IsInstance"
    )
    return pyapi.builder.call(fn, (obj, typ))


def ir_build_datetime(val, builder, *, from_utc: bool = False):
    mod = builder.module
    fnty = ir.FunctionType(ir.PointerType(ir.IntType(8)), [ir.IntType(64)])
    if not from_utc:
        build_datetime_ns_fn = cgutils.get_or_insert_function(
            mod, fnty, "build_datetime_ns"
        )
    else:
        build_datetime_ns_fn = cgutils.get_or_insert_function(
            mod, fnty, "build_datetime_from_utc_ns"
        )
    ptr = builder.call(build_datetime_ns_fn, [val])
    return ptr


def ir_timestamp_nanos(ptr, builder):
    mod = builder.module
    fnty = ir.FunctionType(ir.IntType(64), [ir.PointerType(ir.IntType(8))])
    fn = cgutils.get_or_insert_function(mod, fnty, "timestamp_nanos")
    ptr = builder.call(fn, [ptr])
    return ptr


def ir_local_timestamp_nanos(ptr, builder):
    mod = builder.module
    fnty = ir.FunctionType(ir.IntType(64), [ir.PointerType(ir.IntType(8))])
    fn = cgutils.get_or_insert_function(mod, fnty, "local_timestamp_nanos")
    ptr = builder.call(fn, [ptr])
    return ptr


def long_as_ulong(pyapi, numobj):
    fnty = ir.FunctionType(pyapi.ulong, [pyapi.pyobj])
    fn = pyapi._get_function(fnty, name="PyLong_AsUnsignedLong")
    return pyapi.builder.call(fn, [numobj])
