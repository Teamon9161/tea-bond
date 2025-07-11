from llvmlite import ir
from numba import types
from numba.core import cgutils, utils
from numba.cpython.hashing import _Py_hash_t
from numba.extending import (
    as_numba_type,
    box,
    lower_builtin,
    make_attribute_wrapper,
    models,
    overload_attribute,
    overload_method,
    register_model,
    type_callable,
    typeof_impl,
)
from .nb_date import date_type

from pybond import TfEvaluator
from pybond.ffi import (
    # create_tf_evaluator,
    # create_tf_evaluator_with_reinvest,
    # free_string,
    # free_tf_evaluator,
    tf_evaluator_accrued_interest,
    tf_evaluator_basis_spread,
    # tf_evaluator_bond_code,
    tf_evaluator_bond_ytm,
    tf_evaluator_calc_all,
    tf_evaluator_capital_rate,
    tf_evaluator_carry,
    tf_evaluator_cf,
    tf_evaluator_clean_price,
    tf_evaluator_deliver_accrued_interest,
    tf_evaluator_deliver_cost,
    tf_evaluator_dirty_price,
    tf_evaluator_duration,
    tf_evaluator_f_b_spread,
    # tf_evaluator_future_code,
    tf_evaluator_future_dirty_price,
    tf_evaluator_future_price,
    tf_evaluator_future_ytm,
    # tf_evaluator_get_date,
    # tf_evaluator_get_deliver_date,
    tf_evaluator_irr,
    tf_evaluator_is_deliverable,
    tf_evaluator_net_basis_spread,
    tf_evaluator_reinvest_rate,
    tf_evaluator_remain_cp_num,
    tf_evaluator_remain_cp_to_deliver,
    tf_evaluator_remain_cp_to_deliver_wm,
    tf_evaluator_remain_days_to_deliver,
    # tf_evaluator_update_info,
)



class TfEvaluatorType(types.Type):
    def __init__(self):
        super().__init__(name="TfEvaluator")


tf_evaluator_type = TfEvaluatorType()
as_numba_type.register(TfEvaluator, tf_evaluator_type)


@typeof_impl.register(TfEvaluator)
def typeof_tf_evaluator(val, c):
    return tf_evaluator_type


@type_callable(TfEvaluator)
def type_tf_evaluator(context):
    def typer(
        future,
        bond,
        date=None,
        future_price=float("nan"),
        bond_ytm=float("nan"),
        capital_rate=float("nan"),
        reinvest_rate=None,
    ):
        return tf_evaluator_type

    return typer


@register_model(TfEvaluatorType)
class TfEvaluatorModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("ptr", types.voidptr),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(TfEvaluatorType, "ptr", "ptr")


@lower_builtin(
    TfEvaluator,
    types.string,
    types.string,
    date_type,
    types.float64,
    types.float64,
    types.float64,
)
def impl_tf_evaluator_builder(context, builder, sig, args):
    typ = sig.return_type
    (
        future,
        bond,
        date_val,
        future_price,
        bond_ytm,
        capital_rate,
    ) = args

    # Get string data from Numba strings
    future_code = context.make_helper(builder, types.string, future)
    bond_code = context.make_helper(builder, types.string, bond)
    date = context.make_helper(builder, date_type, date_val)

    # Call create_tf_evaluator
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(
            ir.PointerType(ir.IntType(8)),
            [
                ir.PointerType(ir.IntType(8)),  # future_code_ptr
                ir.IntType(utils.MACHINE_BITS),  # future_code_len
                ir.DoubleType(),  # future_price
                ir.PointerType(ir.IntType(8)),  # bond_code_ptr
                ir.IntType(utils.MACHINE_BITS),  # bond_code_len
                ir.DoubleType(),  # bond_ytm
                ir.DoubleType(),  # capital_rate
                ir.IntType(32),  # year
                ir.IntType(32),  # month
                ir.IntType(32),  # day
            ],
        ),
        name="create_tf_evaluator",
    )

    ptr = builder.call(
        fn,
        [
            future_code.data,
            future_code.length,
            future_price,
            bond_code.data,
            bond_code.length,
            bond_ytm,
            capital_rate,
            date.year,
            date.month,
            date.day,
        ],
    )

    # Create TfEvaluator object
    evaluator = cgutils.create_struct_proxy(typ)(context, builder)
    evaluator.ptr = ptr
    return evaluator._getvalue()


@lower_builtin(
    TfEvaluator,
    types.string,
    types.string,
    date_type,
    types.float64,
    types.float64,
    types.float64,
    types.float64,
)
def impl_tf_evaluator_builder_with_reinvest(context, builder, sig, args):
    typ = sig.return_type
    (
        future_code_val,
        bond_code_val,
        date_val,
        future_price_val,
        bond_ytm_val,
        capital_rate_val,
        reinvest_rate_val,
    ) = args

    # Get string data from Numba strings
    future_code = context.make_helper(builder, types.string, future_code_val)
    bond_code = context.make_helper(builder, types.string, bond_code_val)

    # Get date components
    date = context.make_helper(builder, date_type, date_val)

    # Call create_tf_evaluator_with_reinvest
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(
            ir.PointerType(ir.IntType(8)),
            [
                ir.PointerType(ir.IntType(8)),  # future_code_ptr
                ir.IntType(utils.MACHINE_BITS),  # future_code_len
                ir.DoubleType(),  # future_price
                ir.PointerType(ir.IntType(8)),  # bond_code_ptr
                ir.IntType(utils.MACHINE_BITS),  # bond_code_len
                ir.DoubleType(),  # bond_ytm
                ir.DoubleType(),  # capital_rate
                ir.DoubleType(),  # reinvest_rate
                ir.IntType(32),  # year
                ir.IntType(32),  # month
                ir.IntType(32),  # day
            ],
        ),
        name="create_tf_evaluator_with_reinvest",
    )

    ptr = builder.call(
        fn,
        [
            future_code.data,
            future_code.length,
            future_price_val,
            bond_code.data,
            bond_code.length,
            bond_ytm_val,
            capital_rate_val,
            reinvest_rate_val,
            date.year,
            date.month,
            date.day,
        ],
    )

    # Create TfEvaluator object
    evaluator = cgutils.create_struct_proxy(typ)(context, builder)
    evaluator.ptr = ptr
    return evaluator._getvalue()


# Helper function to create LLVM calls for TfEvaluator methods
def create_tf_evaluator_method_call(method_name, return_type=ir.DoubleType()):
    def impl(context, builder, sig, args):
        evaluator_val = args[0]
        evaluator = cgutils.create_struct_proxy(TfEvaluatorType)(
            context, builder, value=evaluator_val
        )

        fn = cgutils.get_or_insert_function(
            builder.module,
            ir.FunctionType(return_type, [ir.PointerType(ir.IntType(8))]),
            name=method_name,
        )

        result = builder.call(fn, [evaluator.ptr])
        return result

    return impl


# Property getters
@overload_attribute(TfEvaluatorType, "is_deliverable")
def tf_evaluator_attr_is_deliverable(evaluator):
    def impl(evaluator):
        return tf_evaluator_is_deliverable(evaluator.ptr) == 1

    return impl


@overload_attribute(TfEvaluatorType, "bond_ytm")
def tf_evaluator_attr_bond_ytm(evaluator):
    def impl(evaluator):
        return tf_evaluator_bond_ytm(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "future_price")
def tf_evaluator_attr_future_price(evaluator):
    def impl(evaluator):
        return tf_evaluator_future_price(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "capital_rate")
def tf_evaluator_attr_capital_rate(evaluator):
    def impl(evaluator):
        return tf_evaluator_capital_rate(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "reinvest_rate")
def tf_evaluator_attr_reinvest_rate(evaluator):
    def impl(evaluator):
        return tf_evaluator_reinvest_rate(evaluator.ptr)

    return impl


# Calculation methods
@overload_attribute(TfEvaluatorType, "accrued_interest")
def tf_evaluator_method_accrued_interest(evaluator):
    def impl(evaluator):
        return tf_evaluator_accrued_interest(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "deliver_accrued_interest")
def tf_evaluator_method_deliver_accrued_interest(evaluator):
    def impl(evaluator):
        return tf_evaluator_deliver_accrued_interest(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "cf")
def tf_evaluator_method_cf(evaluator):
    def impl(evaluator):
        return tf_evaluator_cf(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "dirty_price")
def tf_evaluator_method_dirty_price(evaluator):
    def impl(evaluator):
        return tf_evaluator_dirty_price(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "clean_price")
def tf_evaluator_method_clean_price(evaluator):
    def impl(evaluator):
        return tf_evaluator_clean_price(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "future_dirty_price")
def tf_evaluator_method_future_dirty_price(evaluator):
    def impl(evaluator):
        return tf_evaluator_future_dirty_price(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "deliver_cost")
def tf_evaluator_method_deliver_cost(evaluator):
    def impl(evaluator):
        return tf_evaluator_deliver_cost(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "basis_spread")
def tf_evaluator_method_basis_spread(evaluator):
    def impl(evaluator):
        return tf_evaluator_basis_spread(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "f_b_spread")
def tf_evaluator_method_f_b_spread(evaluator):
    def impl(evaluator):
        return tf_evaluator_f_b_spread(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "carry")
def tf_evaluator_method_carry(evaluator):
    def impl(evaluator):
        return tf_evaluator_carry(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "net_basis_spread")
def tf_evaluator_method_net_basis_spread(evaluator):
    def impl(evaluator):
        return tf_evaluator_net_basis_spread(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "duration")
def tf_evaluator_method_duration(evaluator):
    def impl(evaluator):
        return tf_evaluator_duration(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "irr")
def tf_evaluator_method_irr(evaluator):
    def impl(evaluator):
        return tf_evaluator_irr(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "future_ytm")
def tf_evaluator_method_future_ytm(evaluator):
    def impl(evaluator):
        return tf_evaluator_future_ytm(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "remain_days_to_deliver")
def tf_evaluator_method_remain_days_to_deliver(evaluator):
    def impl(evaluator):
        return tf_evaluator_remain_days_to_deliver(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "remain_cp_num")
def tf_evaluator_method_remain_cp_num(evaluator):
    def impl(evaluator):
        return tf_evaluator_remain_cp_num(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "remain_cp_to_deliver")
def tf_evaluator_method_remain_cp_to_deliver(evaluator):
    def impl(evaluator):
        return tf_evaluator_remain_cp_to_deliver(evaluator.ptr)

    return impl


@overload_attribute(TfEvaluatorType, "remain_cp_to_deliver_wm")
def tf_evaluator_method_remain_cp_to_deliver_wm(evaluator):
    def impl(evaluator):
        return tf_evaluator_remain_cp_to_deliver_wm(evaluator.ptr)

    return impl


@overload_method(TfEvaluatorType, "calc_all")
def tf_evaluator_method_calc_all(evaluator):
    def impl(evaluator):
        _ = tf_evaluator_calc_all(evaluator.ptr)
        return evaluator

    return impl


@box(TfEvaluatorType)
def box_tf_evaluator(typ, val, c):
    """
    Convert a native TfEvaluator structure to python TfEvaluator.
    """
    evaluator = cgutils.create_struct_proxy(typ)(c.context, c.builder, value=val)

    # Convert the pointer to usize for from_ptr method
    ptr_as_usize = c.builder.ptrtoint(evaluator.ptr, ir.IntType(64))
    ptr_obj = c.pyapi.from_native_value(types.uint64, ptr_as_usize)

    # Get TfEvaluator class and call from_ptr
    class_obj = c.pyapi.unserialize(c.pyapi.serialize_object(TfEvaluator))
    from_ptr_method = c.pyapi.object_getattr_string(class_obj, "from_ptr")

    # Call TfEvaluator.from_ptr(ptr)
    res = c.pyapi.call_function_objargs(from_ptr_method, (ptr_obj,))

    # Clean up Python objects
    c.pyapi.decref(ptr_obj)
    c.pyapi.decref(class_obj)
    c.pyapi.decref(from_ptr_method)

    return res
