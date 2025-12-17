# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tea-Bond is a high-performance Rust library for Chinese bond market quantitative analysis. It provides bond pricing, yield calculations, futures arbitrage analysis, and Python bindings via PyO3.

## Build Commands

```bash
# Build all workspace crates
cargo build

# Build with specific features
cargo build --features "download,pnl,batch"

# Run tests
cargo test

# Build Python bindings (from pybond directory)
cd pybond && maturin develop

# Run Python tests
cd pybond && python -m pytest
```

## Workspace Structure

This is a Cargo workspace with 4 crates:

- **tea-bond** (`tea-bond/`): Core bond calculation library
  - `bond/`: Bond struct and calculations (YTM, duration, accrued interest, clean/dirty price)
  - `future/`: Treasury futures (T, TF, TS contracts) and conversion factors
  - `tf_evaluator/`: Futures-spot arbitrage analysis (basis, IRR, carry, BNOC)
  - `pnl/`: P&L calculation (requires `pnl` feature)
  - Features: `download`, `pnl`, `batch`, `duckdb`

- **tea-calendar** (`tea-calendar/`): Trading calendar library
  - China market calendars: SSE (Shanghai Stock Exchange), IB (Interbank)
  - `Calendar` trait with `is_business_day()` and `find_workday()` methods

- **pybond** (`pybond/`): Python bindings using PyO3/maturin
  - Exports: `Bond`, `Future`, `TfEvaluator` classes
  - Supports Polars expressions and Numba nopython mode
  - Features: `download`, `pnl`, `batch`, `persist`

- **bond-ffi** (`bond-ffi/`): C FFI interface for use with Numba
  - Static library exposing bond calculation functions

## Key Types

- `Bond`: Core bond struct with fields for code, market, coupon type, rates, dates
- `BondYtm`: Bond with associated yield-to-maturity
- `CachedBond`: Bond with cached calculation results
- `Future`: Treasury futures contract (parses codes like "T2409")
- `TfEvaluator`: Futures-spot arbitrage calculator

## Market Conventions

- IB (Interbank): Accrued interest uses actual/actual, "count start, not end"
- SH/SSE/SZE (Exchange): Accrued interest uses actual/365, "count both start and end"
- Conversion factors follow CFFEX (China Financial Futures Exchange) standards

## Rust Edition

Uses Rust 2024 edition with nightly toolchain (see `rust-toolchain.toml`).

## Dependencies

Key external dependencies:
- `chrono`: Date handling
- `tea-time` (from tevec): Time utilities
- `pyo3`/`maturin`: Python bindings
- `tevec`: Vector operations (optional, for batch/pnl features)
