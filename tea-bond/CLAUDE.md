# tea-bond

## Purpose

Core Rust crate for bond pricing, futures delivery evaluation, day-count logic, and trade PnL calculations.

## Stack

- `src/bond/` owns the bond model, metadata IO/download, and yield calculations.
- `src/future/` models treasury futures contracts and deliverability windows.
- `src/tf_evaluator/` calculates delivery basket metrics, basis, IRR, and reinvestment variants.
- `src/pnl/` converts position signals and trades into bond PnL reports.

## Conventions

- Keep generic pricing logic in this crate; FFI/Python conversion belongs in `bond-ffi` or `pybond`.
- Domain modules under `src/` expose compact public APIs through `lib.rs` and module `mod.rs` files.

<!-- zr:files -->
- `Cargo.toml` — Rust project manifest (~278 tok)
- `src/`
  - `day_counter.rs` — pub DayCountRule, Actual, Thirty, Business (~962 tok)
  - `export.rs` — pub compact_str, reqwest, serde_json, calendar, tokio (~41 tok)
  - `future/future_price.rs` — pub FuturePrice, new, with_price (~196 tok)
  - `future/future_type.rs` — pub FutureType, is_deliverable (~923 tok)
  - `future/impls.rs` — impl Future, FuturePrice (~336 tok)
  - `future/mod.rs` — pub FuturePrice, FutureType, Future, new, next_future (~3776 tok)
  - `lib.rs` — pub TfEvaluator, SmallStr (~97 tok)
  - `pnl/fee.rs` — pub Fee, amount (~635 tok)
  - `pnl/mod.rs` — pub Fee, PnlReport, BondTradePnlOpt, calc_bond_trade_pnl (~2219 tok)
  - `pnl/trade_from_signal.rs` — pub Trade, TradeFromPosOpt, trading_from_pos (~1468 tok)
  - `tf_evaluator/evaluator.rs` — pub TfEvaluator, new, new_with_reinvest_rate, is_deliverable, with_deliver_date (~4437 tok)
  - `tf_evaluator/impl_traits.rs` — impl TfEvaluator (~745 tok)
  - `tf_evaluator/mod.rs` — pub TfEvaluator (~2228 tok)
  - `tf_evaluator/update_with_new_info.rs` — pub update_with_new_info (~1240 tok)
  - `utils.rs` — pub month_delta, bisection_find_ytm (~369 tok)
<!-- /zr:files -->
<!-- zr:routing -->
- [src/bond/](src/bond/CLAUDE.md)
<!-- /zr:routing -->
