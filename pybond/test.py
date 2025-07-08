from pybond import TfEvaluator
import datetime


e = TfEvaluator("T2509", 240215, "2025-07-03", 100, 0.02)
assert e.bond_code == "240215"
assert e.future == "T2509"
assert e.future_price == 100
assert e.bond_ytm == 0.02
assert e.date == datetime.date(2025, 7, 3)
