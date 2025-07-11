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
    overload_method,
    register_model,
    type_callable,
    typeof_impl,
    unbox,
)

from pybond.ffi import (
    datetime_add_duration,
    datetime_sub_duration,
    get_datetime_day,
    get_datetime_hour,
    get_datetime_minute,
    get_datetime_month,
    get_datetime_nanosecond,
    get_datetime_second,
    get_datetime_year,
    timestamp_nanos,
)

from .ir_utils import (
    ir_build_datetime,
    ir_isinstance,
    ir_local_timestamp_nanos,
    ir_timestamp_nanos,
)
from .nb_duration import DurationType
from .nb_time import Time


class DateTime:
    def __init__(self, val: int):
        self.ptr = 0
        self.val = int(val)

    @property
    def _pydt(self) -> datetime.datetime:
        return datetime.datetime.fromtimestamp(self.val / 1e9)

    @property
    def time(self) -> datetime.time:
        return self._pydt.time

    @property
    def year(self) -> int:
        return self.pydt.year

    @property
    def month(self) -> int:
        return self._pydt.month

    @property
    def day(self) -> int:
        return self._pydt.day

    @property
    def hour(self) -> int:
        return self._pydt.hour

    @property
    def minute(self) -> int:
        return self._pydt.minute

    @property
    def second(self) -> int:
        return self._pydt.second

    @property
    def nanosecond(self) -> int:
        return int(self.val % 1e9)

    def __str__(self):
        return str(self._pydt)

    def __repr__(self):
        return repr(self._pydt)

    def __ge__(self, other):
        return self.val >= other.val

    def __gt__(self, other):
        return self.val > other.val

    def __lt__(self, other):
        return self.val < other.val

    def __le__(self, other):
        return self.val <= other.val

    def __eq__(self, other):
        return self.val == other.val

    def __ne__(self, other):
        return self.val != other.val


class DateTimeType(types.Type):
    def __init__(self):
        super().__init__(name="DateTime")


datetime_type = DateTimeType()
as_numba_type.register(datetime.datetime, datetime_type)


@typeof_impl.register(datetime.datetime)
@typeof_impl.register(DateTime)
def typeof_index(val, c):
    return datetime_type


@type_callable(DateTime)
def type_datetime(context):
    def typer(val):
        return datetime_type

    return typer


@register_model(DateTimeType)
class DateTimeModel(models.StructModel):
    def __init__(self, dmm, fe_type):
        members = [
            ("ptr", types.voidptr),
            ("val", types.int64),
        ]
        models.StructModel.__init__(self, dmm, fe_type, members)


make_attribute_wrapper(DateTimeType, "ptr", "ptr")
make_attribute_wrapper(DateTimeType, "val", "val")


@lower_builtin(DateTime, types.int64)
@lower_builtin(DateTime, types.float64)
def impl_datetime_builder(context, builder, sig, args):
    typ = sig.return_type
    (val,) = args
    if isinstance(val.type, ir.DoubleType):
        val = builder.fptosi(val, ir.IntType(64))
    ptr = ir_build_datetime(val, builder)
    datetime_struct = cgutils.create_struct_proxy(typ)(context, builder)
    datetime_struct.ptr = ptr
    datetime_struct.val = val
    return datetime_struct._getvalue()


@unbox(DateTimeType)
def unbox_datetime(typ, obj, c):
    """
    Convert a Python datetime object or DateTime object to a native datetime structure.
    """
    # Get datetime.datetime type object
    datetime_type_c = c.pyapi.unserialize(c.pyapi.serialize_object(datetime.datetime))

    # Check if object is instance of datetime.datetime
    is_datetime = ir_isinstance(c.pyapi, obj, datetime_type_c)
    c.pyapi.decref(datetime_type_c)

    val_ptr = cgutils.alloca_once(c.builder, ir.IntType(64))
    ptr_ptr = cgutils.alloca_once(c.builder, ir.PointerType(ir.IntType(8)))

    with c.builder.if_else(is_datetime) as (then, otherwise):
        with then:
            # Handle datetime.datetime
            ts_obj = c.pyapi.call_method(obj, "timestamp")
            val = c.pyapi.float_as_double(ts_obj)
            val = c.builder.fmul(
                val, ir.Constant(ir.DoubleType(), 1e9)
            )  # Convert to nanoseconds
            val = c.builder.fptosi(val, ir.IntType(64))  # Convert to int64
            c.pyapi.decref(ts_obj)
            ptr = ir_build_datetime(val, c.builder, from_utc=True)
            c.builder.store(ptr, ptr_ptr)
            c.builder.store(ir_timestamp_nanos(ptr, c.builder), val_ptr)

        with otherwise:
            # Handle DateTime object
            val_obj = c.pyapi.object_getattr_string(obj, "val")
            val = c.pyapi.long_as_longlong(val_obj)
            c.pyapi.decref(val_obj)
            ptr = ir_build_datetime(val, c.builder, from_utc=False)
            c.builder.store(ptr, ptr_ptr)
            c.builder.store(val, val_ptr)

    ptr = c.builder.load(ptr_ptr)
    val = c.builder.load(val_ptr)

    # Build the datetime struct with the computed timestamp value
    datetime_struct = cgutils.create_struct_proxy(typ)(c.context, c.builder)
    datetime_struct.ptr = ptr
    datetime_struct.val = val

    # Check for errors
    is_error = cgutils.is_not_null(c.builder, c.pyapi.err_occurred())
    return NativeValue(datetime_struct._getvalue(), is_error=is_error)


@box(DateTimeType)
def box_datetime(typ, val, c):
    """
    Convert a native Datetime structure to python datetime.datetime.
    """
    dt = cgutils.create_struct_proxy(typ)(c.context, c.builder, value=val)
    utc_timestamp_val = ir_local_timestamp_nanos(dt.ptr, c.builder)
    val = c.builder.sitofp(utc_timestamp_val, ir.DoubleType())  # Convert to double
    val = c.builder.fdiv(val, ir.Constant(ir.DoubleType(), 1e9))  # Convert to seconds
    # Call datetime.datetime.fromtimestamp()
    val_obj = c.pyapi.float_from_double(val)
    class_obj = c.pyapi.unserialize(c.pyapi.serialize_object(datetime.datetime))
    res = c.pyapi.call_method(class_obj, "fromtimestamp", (val_obj,))
    c.pyapi.decref(val_obj)
    c.pyapi.decref(class_obj)
    return res


@overload_attribute(DateTimeType, "timestamp_nanos")
def get_timestamp_nanos_impl(dt):
    def getter(dt):
        return timestamp_nanos(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "year")
def get_year_impl(dt):
    def getter(dt):
        return get_datetime_year(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "month")
def get_month_impl(dt):
    def getter(dt):
        return get_datetime_month(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "day")
def get_day_impl(dt):
    def getter(dt):
        return get_datetime_day(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "hour")
def get_hour_impl(dt):
    def getter(dt):
        return get_datetime_hour(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "minute")
def get_minute_impl(dt):
    def getter(dt):
        return get_datetime_minute(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "second")
def get_second_impl(dt):
    def getter(dt):
        return get_datetime_second(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "nanosecond")
def get_nanosecond_impl(dt):
    def getter(dt):
        return get_datetime_nanosecond(dt.ptr)

    return getter


@overload_attribute(DateTimeType, "date")
def get_date_impl(dt):
    def getter(dt):
        return datetime.date(dt.year, dt.month, dt.day)

    return getter


@overload_attribute(DateTimeType, "time")
def get_time_impl(dt):
    def getter(dt):
        return Time(dt.hour, dt.minute, dt.second, dt.nanosecond)

    return getter


@overload_method(DateTimeType, "__str__")
def str_datetime(dt):
    """compile this method take a long time"""
    if not isinstance(dt, DateTimeType):
        return

    def impl(dt):
        return f"{str(dt.year).zfill(4)}-{str(dt.month).zfill(2)}-{str(dt.day).zfill(2)} {str(dt.hour).zfill(2)}:{str(dt.minute).zfill(2)}:{str(dt.second).zfill(2)}.{str(dt.nanosecond).zfill(9)[:6]}"

    return impl


@overload(operator.ge)
def impl_ge(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val >= dt2.val

    return impl


@overload(operator.gt)
def impl_gt(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val > dt2.val

    return impl


@overload(operator.lt)
def impl_lt(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val < dt2.val

    return impl


@overload(operator.le)
def impl_le(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val <= dt2.val

    return impl


@overload(operator.eq)
def impl_eq(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val == dt2.val

    return impl


@overload(operator.ne)
def impl_ne(dt, dt2):
    if not isinstance(dt, DateTimeType) or not isinstance(dt, DateTimeType):
        return

    def impl(dt, dt2):
        return dt.val != dt2.val

    return impl


@overload(operator.add)
def impl_datetime_add(val1, val2):
    if isinstance(val1, DateTimeType) and isinstance(val2, DurationType):

        def impl(val1, val2):
            return DateTime(datetime_add_duration(val1.val, val2.ptr))

        return impl
    elif isinstance(val1, DurationType) and isinstance(val2, DateTimeType):

        def impl(val1, val2):
            return DateTime(datetime_add_duration(val2.val, val1.ptr))

        return impl
    return


@overload(operator.sub)
def impl_datetime_sub(val1, val2):
    if isinstance(val1, DateTimeType) and isinstance(val2, DurationType):

        def impl(val1, val2):
            return DateTime(datetime_sub_duration(val1.val, val2.ptr))

        return impl
    return
