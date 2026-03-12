"""
Demo: test Polars Decimal column support in pybond (Bonds & TfEvaluators).
"""

import datetime
from decimal import Decimal

import polars as pl
from pybond.pl import Bonds, TfEvaluators

# ── helpers ──────────────────────────────────────────────────────────────────


def section(title: str):
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print("=" * 60)


# ── 1. Bonds – ytm as Decimal ─────────────────────────────────────────────

section("1. Bonds: ytm column as Decimal → clean_price / dirty_price / duration")

bonds_df = pl.DataFrame(
    {
        "bond": ["250215", "240215", "230015"],
        # cast f64 ytm to Decimal(38, 10) to simulate a Decimal column
        "ytm_f64": [0.021456, 0.020252, 0.019343],
        "date": [
            datetime.date(2025, 12, 21),
            datetime.date(2025, 12, 21),
            datetime.date(2025, 12, 21),
        ],
    }
).with_columns(ytm_dec=(pl.col("ytm_f64") * 100).cast(pl.Decimal(scale=4)))

print("Input DataFrame:")
print(bonds_df)

result_bonds = bonds_df.with_columns(
    # Decimal ytm column → float results expected
    cp_from_dec=Bonds("bond").clean_price("ytm_dec", "date"),
    dp_from_dec=Bonds("bond").dirty_price("ytm_dec", "date"),
    dv_from_dec=Bonds("bond").duration("ytm_dec", "date"),
    # Reference: same calculation from f64 ytm
    cp_from_f64=Bonds("bond").clean_price("ytm_f64", "date"),
)

print("\nResult (Decimal ytm vs f64 ytm – cp values should match):")
print(
    result_bonds.select(
        "bond", "ytm_f64", "cp_from_dec", "cp_from_f64", "dp_from_dec", "dv_from_dec"
    )
)


# ── 2. TfEvaluators – bond_ytm as Decimal ────────────────────────────────

section("2. TfEvaluators: bond_ytm as Decimal → net_basis_spread / clean_price / irr")

tfe_df = pl.DataFrame(
    {
        "future": ["T2509", "T2509", "T2509"],
        "bond": ["240215", "240215", "240215"],
        "date": [
            datetime.date(2025, 5, 11),
            datetime.date(2025, 5, 12),
            datetime.date(2025, 5, 13),
        ],
        "future_price": [103.5, 103.6, 103.4],
        "bond_ytm_f64": [0.0205, 0.0208, 0.0202],
        "capital_rate": [0.016, 0.016, 0.016],
    }
).with_columns(
    bond_ytm_dec=pl.col("bond_ytm_f64").cast(pl.Decimal(precision=38, scale=10))
)

print("Input DataFrame:")
print(tfe_df)

evaluator_dec = TfEvaluators(
    future="future",
    bond="bond",
    date="date",
    future_price="future_price",
    bond_ytm="bond_ytm_dec",  # <── Decimal column
    capital_rate="capital_rate",
)

evaluator_f64 = TfEvaluators(
    future="future",
    bond="bond",
    date="date",
    future_price="future_price",
    bond_ytm="bond_ytm_f64",  # reference f64
    capital_rate="capital_rate",
)

result_tfe = tfe_df.select(
    "date",
    nbs_dec=evaluator_dec.net_basis_spread,
    nbs_f64=evaluator_f64.net_basis_spread,
    cp_dec=evaluator_dec.clean_price,
    cp_f64=evaluator_f64.clean_price,
    irr_dec=evaluator_dec.irr,
    irr_f64=evaluator_f64.irr,
    carry_dec=evaluator_dec.carry,
    carry_f64=evaluator_f64.carry,
)

print("\nResult (Decimal bond_ytm vs f64 – values should match):")
print(result_tfe)


# ── 3. TfEvaluators – future_price as Decimal ────────────────────────────

section("3. TfEvaluators: future_price as Decimal → basis_spread / dirty_price")

tfe_df2 = tfe_df.with_columns(
    future_price_dec=pl.col("future_price").cast(pl.Decimal(precision=38, scale=10))
)

evaluator_fp_dec = TfEvaluators(
    future="future",
    bond="bond",
    date="date",
    future_price="future_price_dec",  # <── Decimal
    bond_ytm="bond_ytm_f64",
    capital_rate="capital_rate",
)

result_fp = tfe_df2.select(
    "date",
    bs_dec=evaluator_fp_dec.basis_spread,
    bs_f64=evaluator_f64.basis_spread,
    fdp_dec=evaluator_fp_dec.future_dirty_price,
    fdp_f64=evaluator_f64.future_dirty_price,
)

print("\nResult (Decimal future_price vs f64):")
print(result_fp)


# ── 4. All-Decimal ────────────────────────────────────────────────────────

section("4. TfEvaluators: both future_price AND bond_ytm as Decimal")

tfe_df3 = tfe_df2.with_columns(
    capital_rate_dec=pl.col("capital_rate").cast(pl.Decimal(precision=38, scale=10))
)

evaluator_all_dec = TfEvaluators(
    future="future",
    bond="bond",
    date="date",
    future_price="future_price_dec",
    bond_ytm="bond_ytm_dec",
    capital_rate="capital_rate_dec",
)

result_all = tfe_df3.select(
    "date",
    nbs=evaluator_all_dec.net_basis_spread,
    cp=evaluator_all_dec.clean_price,
    irr=evaluator_all_dec.irr,
    carry=evaluator_all_dec.carry,
    duration=evaluator_all_dec.duration,
    f_b_spread=evaluator_all_dec.f_b_spread,
)

print("\nResult (all-Decimal inputs):")
print(result_all)

print("\nAll tests completed successfully.")
