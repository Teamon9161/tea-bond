from datetime import date
from tf_evaluator import TfEvaluator, DiscFYtmAlgo, CarryFYtmAlgo
from bondattrretriev.localjson import BondAttrRetrievalFromLocalJson
from bondattr_retrieve import BondAttrRetrieval, CachedBondAttrRetrieval
from cache import LocalMemoryCache
import pprint

if __name__ == '__main__':

    # 以下为示例
    bondattr_retrieval = BondAttrRetrievalFromLocalJson("../resources/bondattrdumps")
    # 这里仅为示例, 以下代码CachedBondAttrRetrieval可以没有
    bondattr_retrieval = CachedBondAttrRetrieval(LocalMemoryCache(), bondattr_retrieval)

    todaystr = date.today().strftime('%Y-%m-%d')
    # 可使用 dump2localjson预先从Wind把债券基本信息存储到本地/resources/bondattrdumps目录
    bondattr_220003 = bondattr_retrieval.retrieve(todaystr, "220003.IB")
    bondattr_220021 = bondattr_retrieval.retrieve(todaystr, "220021.IB")

    # 远期收益率演示
    tf_evaluator0 = TfEvaluator(evaluating_date="2023-01-04",
                               f_symbol='T2303', f_px=100.5300,
                               b_symbol='220003.IB', b_ytm=0.0285,
                               capital_rate=0.017).with_fytmalgo(CarryFYtmAlgo()).with_bondattr(bondattr_220003).calc()
    pprint.pprint(tf_evaluator0.__dict__)
    pprint.pprint(tf_evaluator0.tb_calc.__dict__)

    # 期间付息为0次
    tf_evaluator0 = TfEvaluator(evaluating_date="2022-10-28",
                               f_symbol='T2212', f_px=101.65,
                               b_symbol='220021.IB', b_ytm=0.026625,
                               capital_rate=0.0199).with_bondattr(bondattr_220021).calc()
    pprint.pprint(tf_evaluator0.__dict__)
    pprint.pprint(tf_evaluator0.tb_calc.__dict__)

    # 期间付息为1次
    tf_evaluator1 = TfEvaluator(evaluating_date="2022-09-09",
                               f_symbol='T2212', f_px=101.39,
                               b_symbol='200006.IB', b_ytm=0.026761,
                               capital_rate=0.2600).with_bondattrretrieval(bondattr_retrieval).calc()
    pprint.pprint(tf_evaluator1.__dict__)
    pprint.pprint(tf_evaluator1.tb_calc.__dict__)
    # 期间付息为2次
    tf_evaluator2 = TfEvaluator(evaluating_date="2022-11-02",
                               f_symbol='T2306', f_px=100.59,
                               b_symbol='220010.IB', b_ytm=0.0272,
                               capital_rate=0.018).with_bondattrretrieval(bondattr_retrieval).calc()
    pprint.pprint(tf_evaluator2.__dict__)
    pprint.pprint(tf_evaluator2.tb_calc.__dict__)

    # ---------------指定利息再投资利率
    # 期间付息为1次
    tf_evaluator1 = TfEvaluator(evaluating_date="2022-09-09",
                               f_symbol='T2212', f_px=101.39,
                               b_symbol='200006.IB', b_ytm=0.026761,
                               capital_rate=0.026, b_inst_reinvest_rate=0.0276).with_bondattrretrieval(bondattr_retrieval).calc()
    pprint.pprint(tf_evaluator1.__dict__)
    pprint.pprint(tf_evaluator1.tb_calc.__dict__)
    # 期间付息为2次
    tf_evaluator2 = TfEvaluator(evaluating_date="2022-11-02",
                               f_symbol='T2306', f_px=100.59,
                               b_symbol='220010.IB', b_ytm=0.0272,
                               capital_rate=0.018, b_inst_reinvest_rate=0.0276).with_bondattrretrieval(bondattr_retrieval).calc()
    pprint.pprint(tf_evaluator2.__dict__)
    pprint.pprint(tf_evaluator2.tb_calc.__dict__)
