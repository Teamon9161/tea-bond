from pybond import Bond, TfEvaluator

bond = Bond("019743.SH")
date = "2024-10-20"
ytm = 1.89 / 100

print(bond.accrued_interest(date))
print(bond.dirty_price(ytm, date))
print(bond.clean_price(ytm, date))
print(bond.duration(ytm, date))
print(bond.macaulay_duration(ytm, date))
