# bond

## Purpose

Bond domain model, yield calculations, cached raw-pointer storage, metadata download, and persistence helpers.

## Stack

- `mod.rs` defines the main `Bond` API and exports bond-related types.
- `download/` contains market-specific metadata fetchers.
- `io/` contains local persistence and Wind/DuckDB row conversion.
- `cached_bond.rs` and `impl_convert.rs` support raw-pointer and ownership conversion for FFI/Python boundaries.

## Conventions

- Keep market-specific metadata download/read logic under `download/` and `io/`; core bond math stays in the main bond module files.
- `download/wind.rs` (feature `wind`, Windows/Linux only) is currently the *only* path `Bond::download` uses on those platforms — it no longer falls back to the public web fetchers (`china_money.rs`, `sse.rs`) on Wind failure; those two files are kept on disk but not `mod`-included, so a Wind error now propagates as a proper `Err` (with the underlying Wind error message) instead of being swallowed and silently degrading. On platforms without Wind (e.g. macOS) `Bond::download` always errors — there is no fallback there either. Only the Wind path ever filled `issue_price`, so restoring the web fetchers as a fallback would need to accept that field being unavailable.
- The Wind session is a process singleton pinned to one worker thread (`with_wind`); never construct `wind_rs::Wind` elsewhere. `wind_download_batch` sends one wss request per `WSS_CHUNK` codes — never loop per code.
- `par_value` means **current outstanding principal**, not issue face value: Wind `latestpar`, wind_sql `b_info_curpar` (falling back to `b_info_par`). Both download paths must stay on this convention.
- Raw pointer ownership helpers in `cached_bond.rs` are used across FFI boundaries, so preserve allocation and free semantics when editing them.
- The Wind terminal must already be logged in on the machine before the first `wind_download` call (`wind_rs`'s documented precondition). On one Windows machine, calling before login (Wind then shows its own QR-login popup) once crashed the whole process with `STATUS_STACK_BUFFER_OVERRUN` instead of returning an `Err`; this looked like instability inside Wind's native client during unauthenticated first-connect, not a bug in `wind.rs`/`wind_rs` — but login is a one-shot per-machine/session state, so it's hard to reproduce deliberately. If a download crashes the process outright, check whether the Wind terminal needs a fresh login (scan the QR code) before digging further.

<!-- zr:files -->
- `bond_ytm.rs` — pub BondYtm, new, try_new, with_ytm, ytm (~270 tok)
- `cached_bond.rs` — pub CachedBond, new, into_raw, as_mut_ptr, from_raw (~931 tok)
- `download/`
  - `china_money.rs` — impl Bond (~1517 tok)
  - `mod.rs` — impl Bond (~330 tok)
  - `sse.rs` — impl Bond (~1388 tok)
  - `wind.rs` — pub wind_download, wind_download_batch (~1991 tok)
- `enums.rs` — pub CouponType, InterestType, Market, BondDayCount (~1053 tok)
- `impl_convert.rs` — impl CachedBond, BondYtm (~471 tok)
- `impl_traits.rs` — impl Bond (~389 tok)
- `io/`
  - `duck.rs` — pub read_duckdb (~936 tok)
  - `mod.rs` — pub free_bond_map, WindSqlRow, default_dir, get_json_save_path, read (~1309 tok)
  - `persist.rs` — pub BondMapType, free_bond_map, read_disk, save_disk (~862 tok)
  - `wind_sql_row.rs` — pub WindSqlRow (~1135 tok)
- `mod.rs` — pub BondYtm, CachedBond, Bond, code, bond_code (~6059 tok)
<!-- /zr:files -->
<!-- zr:routing -->
<!-- /zr:routing -->
