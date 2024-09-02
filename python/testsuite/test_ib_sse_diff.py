"""
本测试文件比较交易所和银行间应计利息差异,使用180019.IB=019601.SH=101819.SZ
在Wind上
若以2022年10月18日为结算日，180019.IB的应计利息为0.606033;019601.SH和101819.SZ的应计利息为0.620712
本测试对BondCalculator进行验证
"""

from bond_calculator import BondCalculator
from prim.prim_bondcalculator import PrimitiveBondCalculator
from fi_objects import BondAttributes, dict2bondattr, Market
from pprint import pprint

if __name__ == '__main__':
    ba = {
        "bond_code": "180019.IB",
        "mkt": "IB",
        "abbr": "18附息国债19",
        "par_value": 100.0,
        "cp_type": "Coupon_Bear",
        "interest_type": "Fixed",
        "cp_rate_1st": 0.0354,
        "base_rate": None,
        "rate_spread": None,
        "inst_freq": 2,
        "carry_date": "2018-08-16",
        "maturity_date": "2028-08-16",
        "day_count": "ACT/ACT"
    }

    bondattr = dict2bondattr(ba)
    bc = BondCalculator.configure('2022-10-18', bondattr.bond_code).with_ytm(0.024312) \
        .with_bondattr(bondattr) \
        .make(PrimitiveBondCalculator)
    bc.calc()
    pprint(bc.inst_accrued)
    # ----------------------
    bondattr.bond_code = "019601.SH"
    bondattr.mkt = Market.SH
    bc = BondCalculator.configure('2022-10-18', bondattr.bond_code).with_ytm(0.024312) \
        .with_bondattr(bondattr) \
        .make(PrimitiveBondCalculator)
    bc.calc()
    pprint(bc.inst_accrued)
