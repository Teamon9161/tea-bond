from llvmlite import ir
from numba import types
from numba.core import cgutils, utils
from numba.cpython.hashing import _Py_hash_t
from numba.extending import (
    as_numba_type,
    box,
    intrinsic,
    lower_builtin,
    make_attribute_wrapper,
    models,
    overload_attribute,
    overload_method,
    register_model,
    type_callable,
    typeof_impl,
)

from pybond import Bond
from pybond.ffi import (
    bond_accrued_interest,
    bond_calc_ytm,
    bond_clean_price,
    bond_coupon_rate,
    bond_dirty_price,
    bond_duration,
)
from pybond.nb.nb_datetime import DateTimeType  # , create_bond

from .nb_date import DateType


class BondType(types.Type):
    def __init__(self):
        super().__init__(name="Bond")


bond_type = BondType()
as_numba_type.register(Bond, bond_type)


@typeof_impl.register(Bond)
def typeof_bond(val, c):
    return bond_type


@type_callable(Bond)
def type_bond(context):
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


@lower_builtin(Bond, types.string)
def impl_bond_builder(context, builder, sig, args):
    typ = sig.return_type
    (val,) = args
    # Get string data from Numba string
    code = context.make_helper(builder, types.string, val)
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(
            ir.PointerType(ir.IntType(8)),
            [ir.PointerType(ir.IntType(8)), ir.IntType(utils.MACHINE_BITS)],
        ),
        name="create_bond",
    )
    ptr = builder.call(fn, [code.data, code.length])
    # Create Bond object
    bond = cgutils.create_struct_proxy(typ)(context, builder)
    bond.ptr = ptr
    return bond._getvalue()


@overload_attribute(BondType, "coupon_rate")
def bond_attr_coupon_rate(bond):
    def impl(bond):
        return bond_coupon_rate(bond.ptr)

    return impl


def ir_get_bond_full_code(ptr, context, builder):
    """根据Bond的指针获取bond的代码(包括交易所信息)"""
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(ir.PointerType(ir.IntType(8)), [ir.PointerType(ir.IntType(8))]),
        name="bond_full_code",
    )
    cstr = builder.call(fn, [ptr])
    # 使用 LLVM strlen
    strlen_fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(ir.IntType(64), [ir.PointerType(ir.IntType(8))]),
        name="strlen",
    )
    length = builder.call(strlen_fn, [cstr])
    uni_str = cgutils.create_struct_proxy(types.unicode_type)(context, builder)
    uni_str.data = cstr
    uni_str.length = length
    uni_str.kind = ir.Constant(ir.IntType(32), 1)  # kind=1 is PY_UNICODE_1BYTE_KIND
    uni_str.is_ascii = ir.Constant(ir.IntType(32), 1)
    uni_str.meminfo = context.get_constant_null(types.voidptr)
    # Set hash to -1 to indicate that it should be computed.
    # We cannot bake in the hash value because of hashseed randomization.
    uni_str.hash = context.get_constant(_Py_hash_t, -1)
    uni_str.parent = cgutils.get_null_value(uni_str.parent.type)
    return uni_str._getvalue()


@intrinsic
def get_bond_full_code(typingctx, bond_ptr):
    """根据Bond的指针获取bond的代码(包括交易所信息)"""

    def codegen(context, builder, sig, args):
        return ir_get_bond_full_code(args[0], context, builder)

    sig = types.unicode_type(bond_ptr)
    return sig, codegen


@overload_attribute(BondType, "full_code")
def get_full_code_attr(bond):
    if isinstance(bond, BondType):

        def impl(bond):
            return get_bond_full_code(bond.ptr)

        return impl


@overload_method(BondType, "duration")
def bond_calc_duration(bond, ytm, date):
    if not isinstance(date, (DateType, DateTimeType)):
        return

    def impl(bond, ytm, date):
        return bond_duration(bond.ptr, ytm, date.year, date.month, date.day)

    return impl


@overload_method(BondType, "accrued_interest")
def bond_calc_accrued_interest(bond, date):
    if not isinstance(date, (DateType, DateTimeType)):
        return

    def impl(bond, date):
        return bond_accrued_interest(bond.ptr, date.year, date.month, date.day)

    return impl


@overload_method(BondType, "dirty_price")
def bond_calc_dirty_price(bond, ytm, date):
    if not isinstance(date, (DateType, DateTimeType)):
        return

    def impl(bond, ytm, date):
        return bond_dirty_price(bond.ptr, ytm, date.year, date.month, date.day)

    return impl


@overload_method(BondType, "clean_price")
def bond_calc_clean_price(bond, ytm, date):
    if not isinstance(date, (DateType, DateTimeType)):
        return

    def impl(bond, ytm, date):
        return bond_clean_price(bond.ptr, ytm, date.year, date.month, date.day)

    return impl


@overload_method(BondType, "calc_ytm_with_price")
def bond_ytm_with_price(bond, dirty_price, date):
    if not isinstance(date, (DateType, DateTimeType)):
        return

    def impl(bond, dirty_price, date):
        return bond_calc_ytm(bond.ptr, dirty_price, date.year, date.month, date.day)

    return impl


@box(BondType)
def box_bond(typ, val, c):
    """
    Convert a native Bond structure to python Bond.
    """
    bond = cgutils.create_struct_proxy(typ)(c.context, c.builder, value=val)
    bond_code = ir_get_bond_full_code(bond.ptr, c.context, c.builder)
    # Call Bond new to create a new Bond object
    val_obj = c.pyapi.from_native_value(types.unicode_type, bond_code)
    class_obj = c.pyapi.unserialize(c.pyapi.serialize_object(Bond))
    res = c.pyapi.call_function_objargs(class_obj, (val_obj,))
    c.pyapi.decref(val_obj)
    c.pyapi.decref(class_obj)
    return res
