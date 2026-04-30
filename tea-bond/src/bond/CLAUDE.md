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
- Raw pointer ownership helpers in `cached_bond.rs` are used across FFI boundaries, so preserve allocation and free semantics when editing them.

<!-- zr:files -->
- `bond_ytm.rs` — pub BondYtm, new, try_new, with_ytm, ytm (~257 tok)
- `cached_bond.rs` — pub CachedBond, new, into_raw, as_mut_ptr, from_raw (~901 tok)
- `download/`
  - `china_money.rs` — impl Bond (~1479 tok)
  - `mod.rs` — impl Bond (~174 tok)
  - `sse.rs` — impl Bond (~1348 tok)
- `enums.rs` — pub CouponType, InterestType, Market, BondDayCount (~1017 tok)
- `impl_convert.rs` — impl CachedBond, BondYtm (~448 tok)
- `impl_traits.rs` — impl Bond (~375 tok)
- `io/`
  - `duck.rs` — pub read_duckdb (~753 tok)
  - `mod.rs` — pub free_bond_map, WindSqlRow, default_dir, get_json_save_path, read (~1272 tok)
  - `persist.rs` — pub BondMapType, free_bond_map, read_disk, save_disk (~836 tok)
  - `wind_sql_row.rs` — pub WindSqlRow (~1028 tok)
- `mod.rs` — pub BondYtm, CachedBond, Bond, code, bond_code (~5903 tok)
<!-- /zr:files -->
<!-- zr:routing -->
<!-- /zr:routing -->
