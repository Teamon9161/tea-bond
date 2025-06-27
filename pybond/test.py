from pybond import Bond

data = {
    "bond_code": "123456.IB",
    "abbr": "123456",
    "cp_rate": 0.01,
    "inst_freq": 2,
    "carry_date": "2021-01-01",
    "maturity_date": "2024-01-01",
}

bond = Bond.from_json(data)
