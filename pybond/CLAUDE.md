# pybond

## Purpose

PyO3 crate that exposes `tea-bond` pricing, futures, calendars, persistence, and Polars plugin functions to Python.

## Stack

- `src/` implements the compiled Python extension and Polars plugin bindings in Rust.
- `pybond/` contains the Python package wrappers, FFI helpers, Numba integrations, and type stubs.
- `tests/` covers Python-facing behavior, including decimal handling.

## Conventions

- Keep Rust-facing Python APIs aligned with `pybond/pybond/pybond.pyi` and the wrapper modules under `pybond/pybond/`.
- Build this package from the repository root with `cd pybond && maturin develop --release`.

<!-- zr:files -->
- `.gitignore` —  (~175 tok)
- `Cargo.toml` — Rust project manifest (~345 tok)
- `pyproject.toml` — Python project manifest (~102 tok)
- `src/`
  - `batch_eval.rs` — [derive(Deserialize)] (~6565 tok)
  - `bond.rs` — pub PyBond, download_bond, code, set_code, set_bond_code (~3615 tok)
  - `calendar.rs` — pub Ib, Sse (~269 tok)
  - `future.rs` — pub PyFuture, new (~1281 tok)
  - `lib.rs` — pub get_version (~248 tok)
  - `persist.rs` — pub update_info_from_wind_sql_df (~1293 tok)
  - `pnl.rs` — pub pnl_report_vec_to_series, PyBondTradePnlOpt (~3631 tok)
  - `tf_evaluator.rs` — pub PyTfEvaluator, new, from_ptr (~4711 tok)
  - `utils.rs` — pub extract_date, extract_date2, get_future, get_bond (~715 tok)
- `tests/`
  - `nb_test.py` — def test_bond, test_evaluator (~218 tok)
  - `test_decimal.py` — def section (~1317 tok)
<!-- /zr:files -->
<!-- zr:routing -->
- [pybond/](pybond/CLAUDE.md)
<!-- /zr:routing -->
