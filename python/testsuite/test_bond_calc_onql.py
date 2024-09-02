from cache import LocalMemoryCache
from bond_calculator import BondCalculator
from bondattr_retrieve import CachedBondAttrRetrieval
from wind.wind_context import open_wind_context
from wind.wind_bondattr import BondAttrRetrievalOnWind
from ql.ql_bondcalculator import BondCalculatorOnQL
from pprint import pprint

if __name__ == '__main__':
    local_cache = LocalMemoryCache()

    calculating_date = '2022-11-18'
    bond_code = '220012.IB'
    bond_attributes = None
    # begin: 这部分从Wind取债券基本要素的代码，以后如果移植到服务器上可以用另一版的bondattr_retrieval
    with open_wind_context() as w_ctx:
        # 创建 bondattr_retrieval或者直接得到bondattr
        bondattr_retrieval = BondAttrRetrievalOnWind(w_ctx)
        decorated_bondattr_retrieval = CachedBondAttrRetrieval(local_cache, bondattr_retrieval)
        bond_attributes = decorated_bondattr_retrieval.retrieve(calculating_date, bond_code)
    # end

    # 根据ytm计算价格
    print(f">>{calculating_date}, {bond_code}")
    ql_b_calculator_1 = BondCalculator.configure(calculating_date=calculating_date,
                                                 bond_code=bond_code) \
        .with_ytm(0.0279) \
        .with_bondattr(bond_attributes) \
        .make(BondCalculatorOnQL)
    ql_b_calculator_1.calc()
    pprint(ql_b_calculator_1.__dict__)
    print("-------------------------")
    # 根据净价计算ytm和全价
    print(f">>{calculating_date}, {bond_code}")
    ql_b_calculator_2 = BondCalculator.configure(calculating_date=calculating_date,
                                                 bond_code=bond_code) \
        .with_clean_price(99.7535) \
        .with_bondattr(bond_attributes) \
        .make(BondCalculatorOnQL)
    ql_b_calculator_2.calc()
    pprint(ql_b_calculator_2.__dict__)
    print("-------------------------")
    # 根据全价计算ytm和净价, 这里演示一下with_bondattr的用法
    print(f">>{calculating_date}, {bond_code}")
    ql_b_calculator_3 = BondCalculator.configure(calculating_date='2022-11-18',
                                                 bond_code='220012.IB') \
        .with_dirty_price(100.9288) \
        .with_bondattr(bond_attributes) \
        .make(BondCalculatorOnQL)
    ql_b_calculator_3.calc()
    pprint(ql_b_calculator_3.__dict__)
