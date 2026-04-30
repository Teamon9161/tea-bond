# pybond

## Purpose

Python wrapper package for the compiled `_pybond` extension, including pandas/Polars adapters, FFI bindings, Numba models, and PnL helpers.

## Stack

- `bond.py`, `pd.py`, `pl.py`, and `pnl.py` provide the public Python convenience APIs.
- `ffi/` wraps the C FFI library with `ctypes` for lower-level access.
- `nb/` defines Numba extension types and models for bond, datetime, duration, and evaluator objects.
- `pybond.pyi` is the public type-stub contract for the compiled extension and wrappers.

## Conventions

- Keep high-level wrappers thin over the Rust extension and FFI layer; shared expression/plugin helpers live in `polars_utils.py`.
- Update `pybond.pyi` when public Python-facing classes, methods, or function signatures change.

<!-- zr:files -->
- `__init__.py` — class TfEvaluator, def update_info (~273 tok)
- `bond.py` — class Bond (~1783 tok)
- `download.py` — def save_json, get_interest_type, get_payment_type, fetch_symbols, login (~1037 tok)
- `ffi/`
  - `__init__.py` —  (~23 tok)
  - `bond.py` — create_bond, free_bond, bond_coupon_rate, bond_full_code, bond_calc_ytm (~396 tok)
  - `datetime.py` — build_datetime_ns, build_datetime_from_utc_ns, local_timestamp_nanos, timestamp_nanos, utc_timestamp_to_local (~497 tok)
  - `duration.py` — parse_duration, datetime_sub_datetime, datetime_add_duration, datetime_sub_duration (~167 tok)
  - `evaluators.py` — create_tf_evaluator, create_tf_evaluator_with_reinvest, free_tf_evaluator, tf_evaluator_is_deliverable, tf_evaluator_bond_code (~1718 tok)
  - `lib.py` — lib (~43 tok)
- `nb/`
  - `__init__.py` — from datetime import date, datetime, time (~186 tok)
  - `ir_utils.py` — def ir_isinstance, ir_build_datetime, ir_timestamp_nanos, ir_local_timestamp_nanos, long_as_ulong (~389 tok)
  - `nb_bond.py` — class BondType, BondModel, def typeof_bond, type_bond, impl_bond_builder (~1465 tok)
  - `nb_date.py` — class DateType, DateModel, def typeof_datetime_date, type_date, datetime_date_constructor_u32 (~1405 tok)
  - `nb_datetime.py` — class DateTime, DateTimeType, DateTimeModel, def typeof_index, type_datetime (~2566 tok)
  - `nb_duration.py` — class Duration, DurationType, DurationModel, def typeof_index, type_datetime (~409 tok)
  - `nb_evaluators.py` — class TfEvaluatorType, TfEvaluatorModel, def typeof_tf_evaluator, type_tf_evaluator, impl_tf_evaluator_builder (~3746 tok)
  - `nb_time.py` — class Time, TimeType, TimeModel, def typeof_datetime_time, type_time (~2162 tok)
- `pd.py` — class TfEvaluators, Bonds (~3995 tok)
- `pl.py` — class TfEvaluators, Bonds (~4102 tok)
- `pnl.py` — class Fee, FeeZero, PercentFee, QtyFee, TradeFee (~2512 tok)
- `polars_utils.py` — def parse_into_expr, register_plugin, parse_version (~649 tok)
- `pybond.pyi` — class Bond, Future, TfEvaluator (~3275 tok)
<!-- /zr:files -->
<!-- zr:routing -->
<!-- /zr:routing -->
