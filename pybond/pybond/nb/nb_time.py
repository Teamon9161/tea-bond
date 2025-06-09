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
    overload_attribute,
    # overload_method,
    register_model,
    type_callable,
    typeof_impl,
    unbox,
)

from .ir_utils import ir_isinstance, long_as_ulong


class Time:
    def __init__(self, hour, minute, second, nanosecond):
        # secs = hour * 3600 + minute * 60 + sec
        # self.val = int(secs * 1e9 + nsecs)
        self.hour = hour
        self.minute = minute
        self.second = second
        self.nanosecond = nanosecond


class TimeType(types.Type):
    def __init__(self):
        super().__init__(name="Time")


time_type = TimeType()
as_numba_type.register(datetime.time, time_type)


@typeof_impl.register(datetime.time)
@typeof_impl.register(Time)
def typeof_datetime_time(val, c):
    return time_type


@type_callable(datetime.time)
@type_callable(Time)
def type_time(context):
    def typer(hour, minute, second, nanosecond=0):
        return time_type

    return typer


@register_model(TimeType)
class TimeModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("hour", types.uint32),
            ("minute", types.uint32),
            ("second", types.uint32),
            ("nanosecond", types.uint32),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(TimeType, "hour", "hour")
make_attribute_wrapper(TimeType, "minute", "minute")
make_attribute_wrapper(TimeType, "second", "second")
make_attribute_wrapper(TimeType, "nanosecond", "nanosecond")


@lower_builtin(Time, types.uint32, types.uint32, types.uint32, types.uint32)
@lower_builtin(Time, types.int32, types.int32, types.int32, types.int32)
def time_constructor_u32(context, builder, sig, args):
    time_type = sig.return_type
    time = cgutils.create_struct_proxy(time_type)(context, builder)
    time.hour = args[0]
    time.minute = args[1]
    time.second = args[2]
    time.nanosecond = args[3]
    return time._getvalue()


@lower_builtin(datetime.time, types.uint32, types.uint32, types.uint32, types.uint32)
@lower_builtin(datetime.time, types.int32, types.int32, types.int32, types.int32)
def datetime_time_constructor_u32(context, builder, sig, args):
    time_type = sig.return_type
    time = cgutils.create_struct_proxy(time_type)(context, builder)
    time.hour = args[0]
    time.minute = args[1]
    time.second = args[2]
    # the last param of datetime time is microsecond, but we store nanosecond
    time.nanosecond = builder.mul(args[3], ir.Constant(ir.IntType(32), 1000))
    return time._getvalue()


@lower_builtin(Time, types.int64, types.int64, types.int64, types.int64)
def time_constructor_i64(context, builder, sig, args):
    time_type = sig.return_type
    time = cgutils.create_struct_proxy(time_type)(context, builder)
    time.hour = builder.trunc(args[0], ir.IntType(32))
    time.minute = builder.trunc(args[1], ir.IntType(32))
    time.second = builder.trunc(args[2], ir.IntType(32))
    time.nanosecond = builder.trunc(args[3], ir.IntType(32))
    return time._getvalue()


@lower_builtin(datetime.time, types.int64, types.int64, types.int64, types.int64)
def datetime_time_constructor_i64(context, builder, sig, args):
    time_type = sig.return_type
    time = cgutils.create_struct_proxy(time_type)(context, builder)
    time.hour = builder.trunc(args[0], ir.IntType(32))
    time.minute = builder.trunc(args[1], ir.IntType(32))
    time.second = builder.trunc(args[2], ir.IntType(32))
    nanosecond = builder.mul(args[3], ir.Constant(ir.IntType(64), 1000))
    time.nanosecond = builder.trunc(nanosecond, ir.IntType(32))
    return time._getvalue()


@lower_builtin(datetime.time, types.int64, types.int64, types.int64)
@lower_builtin(Time, types.int64, types.int64, types.int64)
def time_constructor_i64_hms(context, builder, sig, args):
    time_type = sig.return_type
    time = cgutils.create_struct_proxy(time_type)(context, builder)
    time.hour = builder.trunc(args[0], ir.IntType(32))
    time.minute = builder.trunc(args[1], ir.IntType(32))
    time.second = builder.trunc(args[2], ir.IntType(32))
    time.nanosecond = ir.Constant(ir.IntType(32), 0)
    return time._getvalue()


@unbox(TimeType)
def unbox_time(typ, obj, c):
    time_type_c = c.pyapi.unserialize(c.pyapi.serialize_object(datetime.time))
    is_time = ir_isinstance(c.pyapi, obj, time_type_c)
    c.pyapi.decref(time_type_c)

    hour_obj = c.pyapi.object_getattr_string(obj, "hour")
    hour = c.builder.trunc(long_as_ulong(c.pyapi, hour_obj), ir.IntType(32))
    minute_obj = c.pyapi.object_getattr_string(obj, "minute")
    minute = c.builder.trunc(long_as_ulong(c.pyapi, minute_obj), ir.IntType(32))
    second_obj = c.pyapi.object_getattr_string(obj, "second")
    second = c.builder.trunc(long_as_ulong(c.pyapi, second_obj), ir.IntType(32))
    c.pyapi.decref(hour_obj)
    c.pyapi.decref(minute_obj)
    c.pyapi.decref(second_obj)

    nsecs_ptr = cgutils.alloca_once(c.builder, ir.IntType(32))
    with c.builder.if_else(is_time) as (then, otherwise):
        with then:
            msecs_obj = c.pyapi.object_getattr_string(obj, "microsecond")
            msecs = long_as_ulong(c.pyapi, msecs_obj)
            nsecs = c.builder.mul(msecs, ir.Constant(c.pyapi.ulong, 1000))
            nsecs = c.builder.trunc(nsecs, ir.IntType(32))
            c.pyapi.decref(msecs_obj)
            c.builder.store(nsecs, nsecs_ptr)
        with otherwise:
            nsecs_obj = c.pyapi.object_getattr_string(obj, "nanosecond")
            nsecs = long_as_ulong(c.pyapi, nsecs_obj)
            nsecs = c.builder.trunc(nsecs, ir.IntType(32))
            c.pyapi.decref(nsecs_obj)
            c.builder.store(nsecs, nsecs_ptr)
    nanosecond = c.builder.load(nsecs_ptr)
    time = cgutils.create_struct_proxy(typ)(c.context, c.builder)
    time.hour = hour
    time.minute = minute
    time.second = second
    time.nanosecond = nanosecond
    # Check for errors
    is_error = cgutils.is_not_null(c.builder, c.pyapi.err_occurred())
    return NativeValue(time._getvalue(), is_error=is_error)


@box(TimeType)
def box_time(typ, val, c):
    """
    Box a native time object into a Python datetime.time object.
    """
    time = cgutils.create_struct_proxy(typ)(c.context, c.builder, value=val)
    hour_obj = c.pyapi.long_from_unsigned_int(time.hour)
    minute_obj = c.pyapi.long_from_unsigned_int(time.minute)
    second_obj = c.pyapi.long_from_unsigned_int(time.second)
    microsecond = c.builder.udiv(
        time.nanosecond, ir.Constant(time.nanosecond.type, 1000)
    )
    microsecond_obj = c.pyapi.long_from_unsigned_int(microsecond)

    class_obj = c.pyapi.unserialize(c.pyapi.serialize_object(datetime.time))
    time_obj = c.pyapi.call_function_objargs(
        class_obj, (hour_obj, minute_obj, second_obj, microsecond_obj)
    )
    c.pyapi.decref(hour_obj)
    c.pyapi.decref(minute_obj)
    c.pyapi.decref(second_obj)
    c.pyapi.decref(microsecond_obj)
    return time_obj


@overload_attribute(TimeType, "nsecs")
def get_time_impl(dt):
    def getter(dt):
        return int(
            (dt.hour * 3600 + dt.minute * 60 + dt.second) * 1_000_000_000
            + dt.nanosecond
        )

    return getter


@overload(operator.ge)
def impl_ge(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs >= dt2.nsecs

    return impl


@overload(operator.gt)
def impl_gt(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs > dt2.nsecs

    return impl


@overload(operator.le)
def impl_le(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs <= dt2.nsecs

    return impl


@overload(operator.lt)
def impl_lt(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs < dt2.nsecs

    return impl


@overload(operator.eq)
def impl_eq(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs == dt2.nsecs

    return impl


@overload(operator.ne)
def impl_ne(dt, dt2):
    if not isinstance(dt, TimeType) or not isinstance(dt, TimeType):
        return

    def impl(dt, dt2):
        return dt.nsecs != dt2.nsecs

    return impl
