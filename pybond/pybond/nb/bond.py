import numba as nb
from llvmlite import ir
from numba import types
from numba.core import cgutils
from numba.experimental import jitclass
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

from pybond import Bond
from pybond.ffi import bond_duration  # , create_bond
from datetime import date
from .nb_date import DateType

class BondType(types.Type):
    def __init__(self):
        super().__init__(name="DateTime")


bond_type = BondType()
as_numba_type.register(Bond, bond_type)


@typeof_impl.register(Bond)
def typeof_index(val, c):
    return bond_type


@type_callable(Bond)
def type_datetime(context):
    def typer(val):
        return bond_type

    return typer


@register_model(BondType)
class BondModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("ptr", types.voidptr),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(BondType, "ptr", "ptr")

from .ir_utils import ir_build_datetime

@lower_builtin(Bond, types.string)
def impl_bond_builder(context, builder, sig, args):
    typ = sig.return_type
    (val,) = args
    # Get string data from Numba string
    # string = context.make_helper(builder, types.string, val)
    # str_data = string.data  # Pointer to string data
    # str_len = string.length  # Length of string

    # # Allocate space for null-terminated string
    # cstr = builder.alloca(
    #     ir.IntType(8), builder.add(str_len, ir.Constant(ir.IntType(64), 1))
    # )

    # # Copy string data
    # cgutils.memcpy(builder, cstr, str_data, str_len)

    # # Add null terminator
    # null_pos = builder.gep(cstr, [str_len])
    # builder.store(ir.Constant(ir.IntType(8), 0), null_pos)

    # Call FFI function with C string
    # fn = cgutils.get_or_insert_function(
    #     builder.module,
    #     ir.FunctionType(ir.PointerType(ir.IntType(8)), [ir.PointerType(ir.IntType(8))]),
    #     name="create_bond",
    # )
    # ptr = builder.call(fn, [cstr])
    #
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(ir.PointerType(ir.IntType(8)), []),
        name="create_bond",
    )
    ptr = builder.call(fn, [])
    # Create Bond object
    bond = cgutils.create_struct_proxy(typ)(context, builder)
    bond.ptr = ptr
    return bond._getvalue()




@overload_method(BondType, "duration")
def bond_calc_duration(bond, ytm, date):
    if not isinstance(date, DateType):
        return
    def impl(bond, ytm, date):
        return bond_duration(bond.ptr, ytm, date.year, date.month, date.day)

    return impl
