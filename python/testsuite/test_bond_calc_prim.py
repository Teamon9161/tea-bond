from cache import LocalMemoryCache
from bond_calculator import BondCalculator
from bondattr_retrieve import CachedBondAttrRetrieval
from wind.wind_context import open_wind_context
from wind.wind_bondattr import BondAttrRetrievalOnWind
from prim.prim_bondcalculator import PrimitiveBondCalculator
from pprint import pprint

if __name__ == '__main__':
    local_cache = LocalMemoryCache()

    calculating_date = '2022-11-18'
    bond_code = '130005.IB'
    bond_attributes = None
    # begin: 这部分从Wind取债券基本要素的代码，以后如果移植到服务器上可以用另一版的bondattr_retrieval
    # 这部分代码也可以手工实例化一个虚拟的债券,构造一个bond_attributes对象
    with open_wind_context() as w_ctx:
        # 创建 bondattr_retrieval或者直接得到bondattr
        bondattr_retrieval = BondAttrRetrievalOnWind(w_ctx)
        decorated_bondattr_retrieval = CachedBondAttrRetrieval(local_cache, bondattr_retrieval)
        bond_attributes = decorated_bondattr_retrieval.retrieve(calculating_date, bond_code)

    # end

    # 根据ytm计算价格
    print(f">>{calculating_date}, {bond_code}")
    prim_b_calculator_1 = BondCalculator.configure(calculating_date=calculating_date,
                                                   bond_code=bond_code) \
        .with_ytm(0.021818) \
        .with_bondattr(bond_attributes) \
        .make(PrimitiveBondCalculator)
    prim_b_calculator_1.calc()
    pprint(prim_b_calculator_1.__dict__)
    print("-------------------------")

    # 根据净价计算ytm和全价
    print(f">>{calculating_date}, {bond_code}")
    prim_b_calculator_2 = BondCalculator.configure(calculating_date=calculating_date,
                                                   bond_code=bond_code) \
        .with_clean_price(100.3341) \
        .with_bondattr(bond_attributes) \
        .make(PrimitiveBondCalculator)
    prim_b_calculator_2.calc()
    pprint(prim_b_calculator_2.__dict__)
    print("-------------------------")

    # 根据全价计算ytm和净价
    print(f">>{calculating_date}, {bond_code}")
    prim_b_calculator_3 = BondCalculator.configure(calculating_date=calculating_date,
                                                   bond_code=bond_code) \
        .with_dirty_price(101.1854) \
        .with_bondattr(bond_attributes) \
        .make(PrimitiveBondCalculator)
    prim_b_calculator_3.calc()
    pprint(prim_b_calculator_3.__dict__)
