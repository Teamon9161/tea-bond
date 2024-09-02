from cache import *
from wind.wind_context import open_wind_context
from wind.wind_bondattr import *
from wind.wind_bondcalculator import *
from bondattr_retrieve import CachedBondAttrRetrieval
from pprint import pprint

if __name__ == '__main__':
    local_cache = LocalMemoryCache()

    with open_wind_context() as w_ctx:
        # 创建 bondattr_retrieval或者直接得到bondattr
        bondattr_retrieval = BondAttrRetrievalOnWind(w_ctx)
        decorated_bondattr_retrieval = CachedBondAttrRetrieval(local_cache, bondattr_retrieval)

        # 通过producer创建BondCalculator实例,基于Wind API实现的BondCalculator需要用到WindContext对象
        # 根据ytm计算价格
        calculating_date = '2022-11-18'
        bond_code = '220012.IB'
        print(f">>{calculating_date}, {bond_code}")
        w_b_calculator_1 = BondCalculator.configure(calculating_date=calculating_date,
                                                    bond_code=bond_code) \
            .with_ytm(0.0280) \
            .with_bondattr_retrieval(decorated_bondattr_retrieval) \
            .make(BondCalculatorOnWind, w_ctx=w_ctx)
        w_b_calculator_1.calc()
        pprint(w_b_calculator_1.__dict__)
        print("-------------------------")
        # 根据净价计算ytm和全价
        print(f">>{calculating_date}, {bond_code}")
        w_b_calculator_2 = BondCalculator.configure(calculating_date=calculating_date,
                                                    bond_code=bond_code) \
            .with_clean_price(99.6940) \
            .with_bondattr_retrieval(decorated_bondattr_retrieval) \
            .make(BondCalculatorOnWind, w_ctx=w_ctx)
        w_b_calculator_2.calc()
        pprint(w_b_calculator_2.__dict__)
        print("-------------------------")
        # 根据全价计算ytm和净价, 这里演示一下with_bondattr的用法
        print(f">>{calculating_date}, {bond_code}")
        w_b_calculator_3 = BondCalculator.configure(calculating_date=calculating_date,
                                                    bond_code=bond_code) \
            .with_dirty_price(101.8693) \
            .with_bondattr(decorated_bondattr_retrieval.retrieve(calculating_date, bond_code)) \
            .make(BondCalculatorOnWind, w_ctx=w_ctx)
        w_b_calculator_3.calc()
        pprint(w_b_calculator_3.__dict__)
