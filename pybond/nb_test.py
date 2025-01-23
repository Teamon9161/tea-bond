from llvmlite import ir
from numba import types, njit
from numba.core import cgutils
from numba.extending import (
    NativeValue,
    as_numba_type,
    box,
    lower_builtin,
    make_attribute_wrapper,
    models,
    overload,
    overload_attribute,
    overload_method,
    register_model,
    type_callable,
    typeof_impl,
    unbox,
)
from pybond.nb import DateTime
from pybond.ffi import *

# #[no_mangle]
# pub extern "C" fn build_datetime_ns(val: i64) -> *mut c_void {
#     let dt = Utc.timestamp_nanos(val).naive_utc();
#     Box::into_raw(Box::new(dt)) as *mut c_void
# }

import ctypes

# ptr = build_datetime_from_utc_ns(ctypes.c_int64(1234567890))


# def ir_build_datetime(val, builder, *, from_utc: bool = False):
#     mod = builder.module
#     fnty = ir.FunctionType(ir.PointerType(ir.IntType(8)), [ir.IntType(64)])
#     if not from_utc:
#         build_datetime_ns_fn = cgutils.get_or_insert_function(
#             mod, fnty, "build_datetime_ns"
#         )
#     else:
#         build_datetime_ns_fn = cgutils.get_or_insert_function(
#             mod, fnty, "build_datetime_from_utc_ns"
#         )
#     ptr = builder.call(build_datetime_ns_fn, [val])
#     return ptr

# @lower_builtin(DateTime, types.int64)
# @lower_builtin(DateTime, types.float64)
# def impl_datetime_builder(context, builder, sig, args):
#     typ = sig.return_type
#     (val,) = args
#     if isinstance(val.type, ir.DoubleType):
#         val = builder.fptosi(val, ir.IntType(64))

#     _ptr = ir_build_datetime(val, builder) # segmant fault
#     ptr = builder.inttoptr(ir.Constant(ir.IntType(64), 0), ir.PointerType(ir.IntType(8))) # success
#     datetime_struct = cgutils.create_struct_proxy(typ)(context, builder)
#     datetime_struct.ptr = ptr
#     datetime_struct.val = val
#     return datetime_struct._getvalue()


@njit
def test():
    # t = Test(1249512)
    t = DateTime(12401204)

test()