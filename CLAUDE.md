# tea-bond

## Stack

Rust workspace (edition 2024), 4 crates:

| Crate | Role |
|-------|------|
| `tea-calendar` | Chinese market trading calendar (IB / SSE / CFFEX / SZE) |
| `tea-bond` | Core bond & futures pricing — YTM, duration, basis, IRR, PnL |
| `bond-ffi` | C FFI static lib wrapping tea-bond (consumed by pybond) |
| `pybond` | PyO3 + Polars plugin Python package |

Dependencies flow: `tea-calendar` ← `tea-bond` ← `bond-ffi` ← `pybond`.

Python side uses PyO3 for Rust↔Python bridging, with a Polars plugin layer for vectorized operations.

## Critical Rules

- Build Python `pybond` with `cd pybond && maturin develop --release`.

## Supported Extraction

Rust, Python, TOML, and Markdown are supported by zrouter summaries and outlines.

## Code Navigation

Before reading unfamiliar files:
1. Check the loaded/current `<!-- zr:files -->` block first.
2. If the file or directory is outside the loaded routed context, run `zrouter query <path> --json`.
3. If the summary is too thin, run `zrouter query <path> --outline`.
4. Only read the file when summary/outline are insufficient.

After adding, deleting, or moving files, run `zrouter refresh <dir>` for the affected directory; if routed CLAUDE.md files were created or removed, run `zrouter refresh <parent>` or `zrouter refresh . -r`.

## Project Memory

Use `.memory/` for durable project knowledge that is not obvious from current code or git history:
- `.memory/decisions.md` — ADRs and do-not-repeat decisions. Check before architectural or behavior changes; append new decisions and mark old ones `[SUPERSEDED]` instead of overwriting.
- `.memory/patterns.md` — reusable code patterns. Check before implementing common functionality; update when a pattern becomes reusable.
- `.memory/inbox.md` — uncertain inferences. Never treat as canonical or promote without user confirmation.

After work, update relevant CLAUDE.md files when conventions or routing context change, and update `.memory/` only for durable decisions/patterns/inferences. Never record secrets, credentials, or user data.

<!-- zr:files -->
- `.gitignore` —  (~34 tok)
- `Cargo.lock` — Rust lockfile (~29139 tok)
- `Cargo.toml` — Rust project manifest (~91 tok)
- `LICENSE` — License file (~265 tok)
- `README.md` — Tea-Bond (~1890 tok)
- `bond-ffi/`
  - `Cargo.toml` — Rust project manifest (~75 tok)
  - `build.rs` —  (~130 tok)
  - `src/bond.rs` — pub create_bond, free_bond, bond_coupon_rate, bond_full_code, bond_calc_ytm (~639 tok)
  - `src/datetime.rs` — pub FfiDateTime, build_datetime_ns, build_datetime_from_utc_ns, free_datetime, get_datetime_hour (~673 tok)
  - `src/duration.rs` — pub parse_duration, datetime_sub_datetime, datetime_add_duration, datetime_sub_duration (~251 tok)
  - `src/evaluators.rs` — pub create_tf_evaluator, create_tf_evaluator_with_reinvest, free_tf_evaluator, tf_evaluator_is_deliverable, tf_evaluator_accrued_interest (~3606 tok)
  - `src/lib.rs` —  (~16 tok)
  - `src/utils.rs` — pub get_str (~46 tok)
- `rust-toolchain.toml` —  (~13 tok)
- `tea-calendar/`
  - `Cargo.toml` — Rust project manifest (~35 tok)
  - `src/calendars/china/ib.rs` — pub IB (~1038 tok)
  - `src/calendars/china/mod.rs` — pub IB, SSE, China (~291 tok)
  - `src/calendars/china/others.rs` — pub CFFEX, SZE (~129 tok)
  - `src/calendars/china/sse.rs` — pub SSE (~2529 tok)
  - `src/calendars/mod.rs` — pub Calendar (~444 tok)
  - `src/lib.rs` —  (~9 tok)
<!-- /zr:files -->
<!-- zr:routing -->
- [pybond/](pybond/CLAUDE.md)
- [tea-bond/](tea-bond/CLAUDE.md)
<!-- /zr:routing -->
