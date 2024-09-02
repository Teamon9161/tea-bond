import math
from bond_calculator import BondCalculator
from prim.prim_bondcalculator import PrimitiveBondCalculator
from fi_utils import natural_days_between, month_delta
from bondattr_retrieve import BondAttrRetrieval
from tf_td import TDService, tf_tdservice
"""TBond Future Evaluator
author: xueweidong@swhysc.com
created:2022-10-30
"""


class FYtmAlgo:
    """远期收益率(Forward YTM)算法函数对象的公共接口类,
    由于各算法要依赖的数据不同, 但在tf_evaluator都已经具备,
    因此直接把 tf_evaluator作为公共依赖注入,
    各子类的结果也直接更新tf_evaluator即可

    基于Wind的版本在wind包中实现
    """

    def set_tf_eval(self, tf_evaluator):
        """该方法由tf_evaluator的with_fytmalgo方法调用, tf_evaluator将自身的引用传递给FYtmAlgo实例"""
        self.tf_evaluator = tf_evaluator


class DiscFYtmAlgo(FYtmAlgo):
    """折现率方法(Discount Method)计算远期收益率"""

    def __init__(self, disc_rate):
        self.disc_rate = disc_rate

    def __call__(self, *args, **kwargs):
        """这里的全价是为了计算远期收益的中间变量,与债券计算器的结果不同"""
        # 全价 = 国债期货价格 * 转换因子 + 国债期货交割时券的应计利息
        tmp_dirty_price = self.tf_evaluator.cf * self.tf_evaluator.f_px + self.tf_evaluator.deliver_inst_accrued
        # 全价折现
        disc_dirty = tmp_dirty_price * math.exp(0 - self.disc_rate * self.tf_evaluator.remaining2dlv_days / 365)
        # 由全价计算YTM
        tb_calc = BondCalculator.configure(calculating_date=self.tf_evaluator.evaluating_date,
                                                bond_code=self.tf_evaluator.b_symbol)\
            .with_dirty_price(disc_dirty)\
            .with_bondattr(self.tf_evaluator.tb_calc.bond_attributes)\
            .make(PrimitiveBondCalculator)
        tb_calc.calc()
        # 将YTM的值更新到tf_evaluator
        self.tf_evaluator.f_ytm = tb_calc.ytm


class CarryFYtmAlgo(FYtmAlgo):
    """Carry方法计算远期收益率"""
    def __call__(self, *args, **kwargs):
        tmp_clean_price = self.tf_evaluator.cf * self.tf_evaluator.f_px + self.tf_evaluator.carry
        tmp_dirty = tmp_clean_price + self.tf_evaluator.deliver_inst_accrued

        tb_calc = BondCalculator.configure(calculating_date=self.tf_evaluator.deliver_date,
                                           bond_code=self.tf_evaluator.b_symbol) \
            .with_dirty_price(tmp_dirty) \
            .with_bondattr(self.tf_evaluator.tb_calc.bond_attributes) \
            .make(PrimitiveBondCalculator)
        tb_calc.calc()

        self.tf_evaluator.f_ytm = tb_calc.ytm


class WindModFYtmAlgo(FYtmAlgo):
    """与此前的Wind方法计算远期收益率（即wind_algos里面的WindFYtmAlgo）相同，只是不再调用Wind的BC1计算器计算YTM"""
    def __call__(self, *args, **kwargs):
        tmp_clean_price = self.tf_evaluator.cf * self.tf_evaluator.f_px

        tb_calc = BondCalculator.configure(calculating_date=self.tf_evaluator.deliver_date,
                                           bond_code=self.tf_evaluator.b_symbol) \
            .with_clean_price(tmp_clean_price) \
            .with_bondattr(self.tf_evaluator.tb_calc.bond_attributes) \
            .make(PrimitiveBondCalculator)
        tb_calc.calc()

        self.tf_evaluator.f_ytm = tb_calc.ytm


class TfEvaluator:

    def __init__(self, evaluating_date: str, f_symbol: str, f_px: float, b_symbol: str, b_ytm: float,
                 capital_rate: float, b_inst_reinvest_rate: float = None):
        """ TfEvaluator构造函数
        evaluating_date:计算日
        f_symbol: 国债期货合约代码
        f_px: 国债期货价格
        b_symbol: 国债债券代码
        b_ytm: 国债YTM，注意单位,0.02代表2%
        capital_rate: 资金成本利率,例如同业拆借利率,注意单位,0.02代表2%
        b_inst_reinvest_rate: 利息再投资收益率,选输,如果为空，则认为等于将要计算的IRR,注意单位,0.02代表2%
        """

        self.evaluating_date: str = evaluating_date
        self.f_symbol: str = f_symbol
        self.f_px: float = f_px
        self.b_symbol: str = b_symbol
        self.b_ytm: float = b_ytm
        self.capital_rate: float = capital_rate
        self.b_inst_reinvest_rate: float = b_inst_reinvest_rate

        self.reset()

    def reset(self):
        # 以下为国债计算器
        self.tb_calc: BondCalculator = None

        # 根据中金所规则计算f_symbol的缴款日
        self.deliver_date: str = None  # 交割日/缴款日
        # 以下字段从Wind wset函数获取
        self.cf: float = None  # 转换因子
        # 以下字段计算
        self.deliver_pre_cp_date: str = None  # 缴款日前一付息日
        self.deliver_next_cp_date: str = None  # 缴款日后一付息日
        self.deliver_inst_accrued: float = None  # 国债期货应计利息
        # 国债面值*票面利率*(国债期货交割缴款日-国债期货缴款日前一付息日)/(国债期货缴款日下一付息日-国债期货缴款日前一付息日)/付息频率
        self.deliver_cost: float = None  # 交割成本
        self.invoice_px: float = None  # 发票价格
        self.f_b_spread: float = None  # 期限价差
        self.discount_term: float = None  # 贴现期限
        self.remaining2dlv_days: int = None  # 剩余天数
        self.remaining2dlv_cp: float = None  # 期间付息
        self.remaining2dlv_cp_wm: float = None  # 加权平均期间付息(weighted mean)
        # 最终需要的字段
        self.basis_spread: float = None  # 基差
        self.carry: float = None  # 持有收益
        self.IRR: float = None  # 隐含回购利率
        self.net_basis_spread: float = None  # 净基差
        # 可交割券(To Deliver)计算服务, 默认使用tf_td模块的全局TDService实例对象
        self.td_service: TDService = tf_tdservice
        # 可选
        self.__f_ytm_algo = None

    def with_fytmalgo(self, f_ytm_algo: FYtmAlgo):
        f_ytm_algo.set_tf_eval(self)
        self.__f_ytm_algo = f_ytm_algo
        return self

    def with_tdservice(self, tdservice: TDService):
        # 可以自行定制传入非默认的对象
        self.td_service = tdservice
        return self

    def with_bondattrretrieval(self, bondattrretrieval:BondAttrRetrieval):
        self.bond_attr = bondattrretrieval.retrieve(self.evaluating_date, self.b_symbol)
        return self

    def with_bondattr(self, bond_attr):
        self.bond_attr = bond_attr
        return self

    def calc(self):

        bond_attributes = self.bond_attr

        self.tb_calc = BondCalculator.configure(calculating_date=self.evaluating_date,
                                                bond_code=self.b_symbol)\
            .with_ytm(self.b_ytm)\
            .with_bondattr(self.bond_attr)\
            .make(PrimitiveBondCalculator)
        self.tb_calc.calc()

        self.deliver_date = self.td_service.get_f_paydate(self.f_symbol).strftime('%Y-%m-%d')
        self.deliver_pre_cp_date, self.deliver_next_cp_date = self.tb_calc.obtain_cp_dates_section(self.deliver_date)
        # 国债期货交割日应计利息=区间付息* (国债期货交割缴款日 - 国债期货缴款日前一付息日) / (国债期货缴款日下一付息日 - 国债期货缴款日前一付息日)
        # 按中金所发布公式, 计算结果四舍五入至小数点后7位
        C = bond_attributes.cp_rate_1st * bond_attributes.par_value / bond_attributes.inst_freq
        org_dlv_inst_accrued = C * natural_days_between(self.deliver_pre_cp_date,
                                                             self.deliver_date) / natural_days_between(
            self.deliver_pre_cp_date, self.deliver_next_cp_date)
        self.deliver_inst_accrued = round(org_dlv_inst_accrued, 7)
        # 转换因子
        b_remaining_cp_times_after_dlv = self.tb_calc.obtain_remaining_cp_number(self.deliver_date)
        # month_num_from_dlv2next_cp = month_delta(self.deliver_date, self.tb_calc.next_cp_date)
        month_num_from_dlv2next_cp = month_delta(self.deliver_date, self.deliver_next_cp_date)
        self.cf = self.td_service.calc_any_cf(f_symbol=self.f_symbol, b_symbol=self.b_symbol,
                                              b_remaining_cp_times_after_dlv=b_remaining_cp_times_after_dlv,
                                              b_cp_rate=bond_attributes.cp_rate_1st,
                                              b_inst_freq=self.tb_calc.bond_attributes.inst_freq,
                                              month_number_to_next_cp_after_dlv=month_num_from_dlv2next_cp)

        # 发票价格=国债期货价格*转换因子+国债期货交割日应计利息
        self.invoice_px = self.f_px * self.cf + self.deliver_inst_accrued
        # 剩余天数=期货缴款日-计算日
        self.remaining2dlv_days = natural_days_between(self.evaluating_date, self.deliver_date)
        # 贴现期限=(期货缴款日-计算日)/365
        self.discount_term = self.remaining2dlv_days / 365
        # 期间付息 剩余天数内的实际付息
        remaining_cp_dates = self.tb_calc.obtain_cp_dates_between(self.evaluating_date, self.deliver_date)
        remaining_cp_times = len(remaining_cp_dates)  # 期间付息次数
        if remaining_cp_times != 0:
            self.remaining2dlv_cp = remaining_cp_times * C
            # 加权平均期间付息,按每个付息日到结算日的年化剩余天数加权的实际付息
            self.remaining2dlv_cp_wm = sum(
                [natural_days_between(d, self.deliver_date) / 365 for d in remaining_cp_dates]) * C
        else:
            # 避免分母中出现0
            self.remaining2dlv_cp = 0
            self.remaining2dlv_cp_wm = 0

        # 交割成本=国债全价-期间付息
        self.deliver_cost = self.tb_calc.dirty_price - self.remaining2dlv_cp
        # 期限价差=发票价格-交割成本
        self.f_b_spread = self.invoice_px - self.deliver_cost
        # 基差=现券净价-期货价格*转换因子
        self.basis_spread = self.tb_calc.clean_price - self.f_px * self.cf
        # 持有收益 = (交割日应计-交易日应计 + 期间付息) + 资金成本率*(加权平均期间付息-债券全价*剩余天数/365)
        self.carry = (
                             self.deliver_inst_accrued - self.tb_calc.inst_accrued + self.remaining2dlv_cp) + self.capital_rate * (
                             self.remaining2dlv_cp_wm - self.tb_calc.dirty_price * self.remaining2dlv_days / 365)
        # self.carry = (self.deliver_inst_accrued - self.tb_calc.inst_accrued + self.remaining2dlv_cp) - (self.capital_rate * self.tb_calc.dirty_price * self.remaining2dlv_days / 365)
        # 净基差 = 基差 - 持有收益
        self.net_basis_spread = self.basis_spread - self.carry
        # 如果定义了利息再投资利率则需要将使用加权平均期间付息乘以该再投资利率
        if self.b_inst_reinvest_rate:
            self.IRR = ((self.invoice_px + self.remaining2dlv_cp + self.remaining2dlv_cp_wm * self.b_inst_reinvest_rate)
                        / self.tb_calc.dirty_price - 1) * 365 / self.remaining2dlv_days
        else:
            # QB: irr=(发票价格+期间付息-现券全价)/(现券全价*剩余天数/365-加权平均期间付息)
            self.IRR = (self.invoice_px + self.remaining2dlv_cp - self.tb_calc.dirty_price) / (
                    self.tb_calc.dirty_price * self.remaining2dlv_days / 365 - self.remaining2dlv_cp_wm)

        if self.__f_ytm_algo:
            self.__f_ytm_algo()
        return self
