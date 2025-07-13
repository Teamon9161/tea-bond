import datetime
import operator

from llvmlite import ir
from numba import types
from numba.core import cgutils
from numba.extending import (
    NativeValue,
    as_numba_type,
    box,
    lower_builtin,
    make_attribute_wrapper,
    models,
    overload,
    register_model,
    type_callable,
    typeof_impl,
    unbox,
)

from .ir_utils import long_as_ulong


class DateType(types.Type):
    def __init__(self):
        super().__init__(name="Date")


date_type = DateType()
as_numba_type.register(datetime.date, date_type)


@typeof_impl.register(datetime.date)
def typeof_datetime_date(val, c):
    return date_type


@type_callable(datetime.date)
def type_date(context):
    def typer(year, month, day):
        return date_type

    return typer


@register_model(DateType)
class DateModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("year", types.uint32),
            ("month", types.uint32),
            ("day", types.uint32),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(DateType, "year", "year")
make_attribute_wrapper(DateType, "month", "month")
make_attribute_wrapper(DateType, "day", "day")


@lower_builtin(datetime.date, types.uint32, types.uint32, types.uint32)
@lower_builtin(datetime.date, types.int32, types.int32, types.int32)
def datetime_date_constructor_u32(context, builder, sig, args):
    date_type = sig.return_type
    date = cgutils.create_struct_proxy(date_type)(context, builder)
    date.year = args[0]
    date.month = args[1]
    date.day = args[2]
    return date._getvalue()


@lower_builtin(datetime.date, types.int64, types.int64, types.int64)
def datetime_date_constructor_i64(context, builder, sig, args):
    date_type = sig.return_type
    date = cgutils.create_struct_proxy(date_type)(context, builder)
    date.year = builder.trunc(args[0], ir.IntType(32))
    date.month = builder.trunc(args[1], ir.IntType(32))
    date.day = builder.trunc(args[2], ir.IntType(32))
    return date._getvalue()


@unbox(DateType)
def unbox_date(typ, obj, c):
    year_obj = c.pyapi.object_getattr_string(obj, "year")
    year = long_as_ulong(c.pyapi, year_obj)
    month_obj = c.pyapi.object_getattr_string(obj, "month")
    month = long_as_ulong(c.pyapi, month_obj)
    day_obj = c.pyapi.object_getattr_string(obj, "day")
    second = long_as_ulong(c.pyapi, day_obj)
    c.pyapi.decref(year_obj)
    c.pyapi.decref(month_obj)
    c.pyapi.decref(day_obj)

    date = cgutils.create_struct_proxy(typ)(c.context, c.builder)
    date.year = c.builder.trunc(year, ir.IntType(32))
    date.month = c.builder.trunc(month, ir.IntType(32))
    date.day = c.builder.trunc(second, ir.IntType(32))
    # Check for errors
    is_error = cgutils.is_not_null(c.builder, c.pyapi.err_occurred())
    return NativeValue(date._getvalue(), is_error=is_error)


@box(DateType)
def box_date(typ, val, c):
    """
    Box a native date object into a Python datetime.date object.
    """
    date = cgutils.create_struct_proxy(typ)(c.context, c.builder, value=val)
    year_obj = c.pyapi.long_from_unsigned_int(date.year)
    month_obj = c.pyapi.long_from_unsigned_int(date.month)
    day_obj = c.pyapi.long_from_unsigned_int(date.day)

    class_obj = c.pyapi.unserialize(c.pyapi.serialize_object(datetime.date))
    date_obj = c.pyapi.call_function_objargs(class_obj, (year_obj, month_obj, day_obj))
    c.pyapi.decref(year_obj)
    c.pyapi.decref(month_obj)
    c.pyapi.decref(day_obj)
    return date_obj


@overload(operator.ge)
def impl_ge(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        if dt.year == dt2.year:
            if dt.month == dt2.month:
                return dt.day >= dt2.day
            else:
                return dt.month >= dt2.month
        else:
            return dt.year >= dt2.year

    return impl


@overload(operator.gt)
def impl_gt(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        if dt.year == dt2.year:
            if dt.month == dt2.month:
                return dt.day > dt2.day
            else:
                return dt.month > dt2.month
        else:
            return dt.year > dt2.year

    return impl


@overload(operator.le)
def impl_le(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        if dt.year == dt2.year:
            if dt.month == dt2.month:
                return dt.day <= dt2.day
            else:
                return dt.month <= dt2.month
        else:
            return dt.year <= dt2.year

    return impl


@overload(operator.lt)
def impl_lt(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        if dt.year == dt2.year:
            if dt.month == dt2.month:
                return dt.day < dt2.day
            else:
                return dt.month < dt2.month
        else:
            return dt.year < dt2.year

    return impl


@overload(operator.eq)
def impl_eq(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        return dt.year == dt2.year and dt.month == dt2.month and dt.day == dt2.day

    return impl


@overload(operator.ne)
def impl_ne(dt, dt2):
    if not isinstance(dt, DateType) or not isinstance(dt, DateType):
        return

    def impl(dt, dt2):
        return dt.year != dt2.year or dt.month != dt2.month or dt.day != dt2.day

    return impl
