from llvmlite import ir
from numba import types
from numba.core import cgutils, utils
from numba.extending import (
    as_numba_type,
    lower_builtin,
    make_attribute_wrapper,
    models,
    register_model,
    type_callable,
    typeof_impl,
)


class Duration:
    def __init__(self, fmt: str):
        self.fmt = fmt


class DurationType(types.Type):
    def __init__(self):
        super().__init__(name="Duration")


duration_type = DurationType()
as_numba_type.register(Duration, duration_type)


@typeof_impl.register(Duration)
def typeof_index(val, c):
    return duration_type


@type_callable(Duration)
def type_datetime(context):
    def typer(val):
        return duration_type

    return typer


@register_model(DurationType)
class DurationModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("ptr", types.voidptr),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(DurationType, "ptr", "ptr")


@lower_builtin(Duration, types.string)
def impl_duration_builder(context, builder, sig, args):
    typ = sig.return_type
    (fmt,) = args
    fmt = context.make_helper(builder, types.string, fmt)
    fn = cgutils.get_or_insert_function(
        builder.module,
        ir.FunctionType(
            ir.PointerType(ir.IntType(8)),
            [ir.PointerType(ir.IntType(8)), ir.IntType(utils.MACHINE_BITS)],
        ),
        name="parse_duration",
    )
    ptr = builder.call(fn, [fmt.data, fmt.length])
    duration = cgutils.create_struct_proxy(typ)(context, builder)
    duration.ptr = ptr
    return duration._getvalue()
