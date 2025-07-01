from pybond import Bond, TfEvaluator

dt = "2025-07-01"
e = TfEvaluator("T2509", 240215, dt, capital_rate=0.019, reinvest_rate=0)

e = e.calc_all()
print(e.update(bond_ytm=0.02).update(future_price=100).calc_all())
