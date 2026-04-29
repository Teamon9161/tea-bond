# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**tea-bond** is a Rust library for Chinese bond market quantitative analysis — bond pricing, YTM, duration, bond-futures basis/IRR/carry, and P&L tracking. It exposes Python bindings via PyO3 (`pybond`).

## Commands

```bash
# Build
cargo build                    # dev build (workspace default members)
cargo build --release
cargo build -p pybond --lib   # Python extension only

# Test
cargo test                     # all Rust tests
cargo test -p tea-bond         # specific crate
cargo test bond_ytm -- --nocapture   # single test by name
cd pybond && python -m pytest  # Python tests

# Lint / Format
cargo clippy
cargo fmt --check
```

## Workspace Layout

Four crates — only `tea-bond`, `tea-calendar`, and `bond-ffi` are default members (pybond is excluded from routine builds):

| Crate | Purpose |
|-------|---------|
| `tea-bond` | Core bond calculation engine |
| `tea-calendar` | Chinese market trading calendars (IB, SSE, CFFEX) |
| `bond-ffi` | C FFI / staticlib wrapper |
| `pybond` | PyO3 Python extension (`cdylib`) |

## Architecture

### tea-bond core modules

**`bond/`** — Main `Bond` struct with pricing methods: `calc_ytm_with_price`, `calc_clean_price_with_ytm`, `calc_dirty_price_with_ytm`, `calc_accrued_interest`, `calc_macaulay_duration`. Key enums: `CouponType`, `Market` (IB/SSE), `BondDayCount`. `BondYtm` bundles a bond with its yield. `CachedBond` memoizes repeated calculations.

**`future/`** — `Future` type parses contract codes (e.g. `"T2409"`), computes maturity and delivery dates. `FutureType` enumerates treasury futures (T/TF/TS/TL).

**`tf_evaluator/`** — Bond-futures basis evaluator ("Tf" = Treasury Futures). `calc_all()` computes in one pass: conversion factor (CF), `basis_spread`, `net_basis_spread`, `irr`, `carry`, `future_ytm`. `update_with_new_info.rs` handles rolling updates when quotes change without reloading bond fundamentals.

**`day_counter.rs`** — Day count convention implementations (Act/Act, Actual/360, 30/360, etc.) used by `Bond` for accrued interest.

**`pnl/`** (feature-gated `pnl`) — Trade P&L, fee handling, signal-to-trade conversion.

### pybond Python bindings

`pybond/src/lib.rs` is the module entry point. Each Rust type gets a `Py*` wrapper (`PyBond`, `PyFuture`, `PyTfEvaluator`). `batch_eval.rs` exposes vectorized Polars expressions for bulk calculations.

### tea-calendar

Provides `next_trade_day`, `is_trade_day`, `offset_trade_days` for IB (interbank) and SSE markets. Handles Chinese holidays and 调休 (makeup workdays).

## Key Conventions

- **Error handling**: `anyhow::Result<T>` everywhere.
- **Dates**: `chrono::NaiveDate` throughout.
- **Strings**: `SmallStr = compact_str::CompactString` for bond codes (avoids heap allocation on short strings).
- **Serialization**: Bonds are constructed from Wind SQL JSON via `serde`.
- **Feature flags** in `tea-bond`: `download` (Wind API fetch), `pnl`, `batch`, `duckdb`. Additional flags in `pybond`: `persist` (Polars), `decimal`.

## Commit Convention

```
[feat]    new feature
[fix]     bug fix
[chore]   maintenance / deps
[api]     public API change
[release] version bump
```
