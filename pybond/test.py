from pybond import TfEvaluator
# bond = Bond(240012)

te = TfEvaluator(
    "T2412.CFE", 240013, "2024-11-19", 106.675, 0.02085, capital_rate=0.018
)
# .calc_all()
print(te.cf)
print(te.net_basis_spread)
te = te.update(107, 0.02075)
print(te)
