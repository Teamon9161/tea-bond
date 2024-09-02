# -*- coding: utf-8 -*-
"""Bond Calculator
 Copyright (C) 2022 www.swhysc.com
 author: fintech.ficc.swhysc
 created:2022-11-19

"""
import datetime
import abc
from typing import Any
from fi_objects import BondAttributes, Market
from bondattr_retrieve import BondAttrRetrieval
from fi_utils import natural_days_between, str2pydate, pydate2str, qldate2str
import QuantLib as ql


class BondCalculator(abc.ABC):

    def __init__(self, calculating_date: str, bond_code: str, bond_attributes: BondAttributes, ytm: float,
                 clean_price: float,
                 dirty_price: float,
                 options: dict):
        self.options = options
        self.calculating_date: str = calculating_date
        self.bond_code: str = bond_code
        self.bond_attributes: BondAttributes = bond_attributes
        self.ytm: float = ytm
        self.clean_price: float = clean_price
        self.dirty_price: float = dirty_price
        self.duration: float = None  # 修正久期

        self.inst_accrued: float = None  # 应计利息
        self.inst_accrued_days: int = None  # 应计利息天数, 为计算日与前一付息日的距离
        self.pre_cp_date: str = ''  # 计算日前一付息日
        self.next_cp: float = None  # 计算日所在息票周期的息票,浮动利率债券的每次coupon都是不一样的
        self.next_cp_date: str = ''  # 计算日下一付息日
        self.present_cp_period_days: int = None  # 计算日所在息票周期的天数
        self.remaining_days2mature: int = None  # 计算日距离到期日的天数
        self.remaining_days2nextcp: int = None  # 计算日距离下一付息日的天数
        self.remaining_cp_num: int = None  # 剩余付息次数
        self.in_final_cp_period: bool = None  # 计算日是否位于最后一个付息周期内

        self._schedule_ql: ql.Schedule = None

    @classmethod
    def new(cls, calculating_date: str, bond_code: str, bond_attributes: BondAttributes, ytm: float, clean_price: float,
            dirty_price: float, options: dict, **kwargs) -> Any:
        inst = cls(calculating_date, bond_code, bond_attributes, ytm, clean_price, dirty_price, options, **kwargs)
        return inst

    def _calc_inst_accrued(self):
        """计算应计利息
        银行间和交易所的计算规则不同,银行间是算头不算尾,而交易所是算头又算尾
        """
        self.pre_cp_date, self.next_cp_date = self.obtain_cp_dates_section(self.calculating_date)
        self.next_cp = self.bond_attributes.cp_rate_1st * self.bond_attributes.par_value / self.bond_attributes.inst_freq
        self.present_cp_period_days = natural_days_between(self.pre_cp_date, self.next_cp_date)
        # todo: 后续再出现市场计算规则的差异，增将差异全都抽象到单独的类里
        if self.bond_attributes.mkt == Market.IB:
            self.inst_accrued_days = natural_days_between(self.pre_cp_date, self.calculating_date)
            self.inst_accrued = self.next_cp * self.inst_accrued_days / self.present_cp_period_days
        else:
            # 交易所算头又算尾
            self.inst_accrued_days = 1 + natural_days_between(self.pre_cp_date, self.calculating_date)
            self.inst_accrued = self.bond_attributes.cp_rate_1st * self.bond_attributes.par_value * self.inst_accrued_days / 365

    def _calc_ref(self):
        """计算参考数据, 仅由calc模板方法调用"""
        self.remaining_days2nextcp = natural_days_between(self.calculating_date, self.next_cp_date)
        self.remaining_days2mature = natural_days_between(self.calculating_date, self.bond_attributes.maturity_date)
        self.remaining_cp_num = self.obtain_remaining_cp_number(self.calculating_date)
        # 如果下一个付息日与到期日的距离在15天以内则认为当前计算日处于最后一个付息周期内,避免到期日和计息截止日不是同一天的情况
        if natural_days_between(self.next_cp_date, self.bond_attributes.maturity_date) < 15:
            self.in_final_cp_period = True
        else:
            self.in_final_cp_period = False

    @abc.abstractmethod
    def _calc_price_with_ytm(self):
        """根据ytm计算净价全价,需要在_calc_inst_accrued之后执行, 仅由calc模板方法调用"""
        pass

    @abc.abstractmethod
    def _calc_ytm_with_price(self):
        """根据全价或净价计算ytm,需要在_calc_inst_accrued之后执行, 仅由calc模板方法调用"""
        pass

    def _calc_duration(self):
        """计算修正久期"""
        pass

    def _pre_calc(self):
        """留给子类扩展预处理"""
        pass

    def _post_calc(self):
        """留给子类扩展后处理"""
        pass

    def calc(self):
        """模板方法"""
        # 使用quantlib的schedule计算该支债券的现金流日历
        self._ensure_qlschedule()
        self._pre_calc()
        # 应计利息的计算ytm无关, 而计算ytm/price需要依赖应计利息
        self._calc_inst_accrued()
        # 计算中间过程数据, 用于分析
        self._calc_ref()
        # 计算价格或ytm
        if self.ytm and not (self.dirty_price or self.clean_price):
            self._calc_price_with_ytm()
        elif (self.dirty_price or self.clean_price) and not self.ytm:
            self._calc_ytm_with_price()
        # 计算久期
        self._calc_duration()
        # 后置处理
        self._post_calc()

    def _ensure_qlschedule(self):
        """使用 QuantLib 构造一个该债券的现金流Schedule"""
        if not self._schedule_ql:
            carryDate_py = str2pydate(self.bond_attributes.carry_date)
            maturityDate_py = str2pydate(self.bond_attributes.maturity_date)
            issueDate_ql = ql.Date(carryDate_py.day, carryDate_py.month, carryDate_py.year)  # 起息日
            maturityDate_ql = ql.Date(maturityDate_py.day, maturityDate_py.month, maturityDate_py.year)  # 到期日
            tenor_ql = ql.Period(ql.Semiannual) if self.bond_attributes.inst_freq == 2 else ql.Period(ql.Annual)  # 付息期限
            calendar_ql = ql.China()  # 日历
            bussinessConvention_ql = ql.Unadjusted  # 遇到假期的调整情况
            dateGeneration = ql.DateGeneration.Backward  # 日期的生成规则（向后推）
            monthEnd = False  # 是否月最后一日
            self._schedule_ql = ql.Schedule(issueDate_ql, maturityDate_ql, tenor_ql, calendar_ql,
                                            bussinessConvention_ql, bussinessConvention_ql, dateGeneration, monthEnd)
        return self._schedule_ql

    def obtain_cp_dates_section(self, any_calc_date: str):
        """使用self._schedule_ql获取日期any_calc_date的上一付息日、下一付息日"""
        self._ensure_qlschedule()
        calc_date_py = datetime.datetime.strptime(any_calc_date, '%Y-%m-%d')
        calc_date_ql = ql.Date(calc_date_py.day, calc_date_py.month, calc_date_py.year)
        for i in range(1, len(self._schedule_ql)):
            # 从1开始获取, 第1个日期为发行日;且如果calc_date_ql比发行日早,说明本次计算输入的at_date不合理
            if self._schedule_ql[i - 1] <= calc_date_ql < self._schedule_ql[i]:
                pre_cp_date = self._schedule_ql[i - 1]
                next_cp_date = self._schedule_ql[i]
                return pydate2str(pre_cp_date.to_date()), pydate2str(next_cp_date.to_date())
        # 如果上面没有找到,则返回一堆空变量
        return None, None

    def obtain_cp_dates_between(self, from_date: str, to_date: str) -> list:
        """使用self._schedule_ql获取from_date到to_date之间的付息次数"""
        self._ensure_qlschedule()
        from_date_py = datetime.datetime.strptime(from_date, '%Y-%m-%d')
        from_date_ql = ql.Date(from_date_py.day, from_date_py.month, from_date_py.year)
        to_date_py = datetime.datetime.strptime(to_date, '%Y-%m-%d')
        to_date_ql = ql.Date(to_date_py.day, to_date_py.month, to_date_py.year)
        ret_tmp = filter(lambda i: from_date_ql < i < to_date_ql, self._schedule_ql)
        return [qldate2str(i) for i in ret_tmp]

    def obtain_remaining_cp_number(self, ref_date: str):
        self._ensure_qlschedule()
        calc_date_py = datetime.datetime.strptime(ref_date, '%Y-%m-%d')
        calc_date_ql = ql.Date(calc_date_py.day, calc_date_py.month, calc_date_py.year)
        return sum(1 for d in self._schedule_ql if calc_date_ql < d)

    @staticmethod
    def configure(calculating_date: str, bond_code: str):
        return BondCalculatorConfigure(calculating_date, bond_code)


class BondCalculatorConfigure:
    """Producer负责准备好经检验的各入参以及bondattr对象,然后实例化BondCalculator"""

    def __init__(self, calculating_date: str, bond_code: str):
        """kwargs用于子类扩展自己所需的参数"""
        self.calculating_date: str = calculating_date
        self.bond_code: str = bond_code
        self.ytm: float = None
        self.clean_price: float = None
        self.dirty_price: float = None
        self.bond_attributes = None
        self.options = {}

    def with_bondattr(self, bond_attributes):
        # with_bondattr和with_bondattr_retrieval二选一
        self.bond_attributes = bond_attributes
        return self

    def with_bondattr_retrieval(self, bondattr_retrieval: BondAttrRetrieval):
        # with_bondattr和with_bondattr_retrieval二选一
        bond_attributes = bondattr_retrieval.retrieve(self.calculating_date, self.bond_code)
        self.bond_attributes = bond_attributes
        return self

    def with_ytm(self, ytm: float):
        self.ytm = ytm
        return self

    def with_clean_price(self, clean_price):
        self.clean_price = clean_price
        return self

    def with_dirty_price(self, dirty_price):
        self.dirty_price = dirty_price
        return self

    def with_options(self, options: dict):
        self.options = options

    def make(self, calculator_cls, **kwargs) -> BondCalculator:
        # 这里不能声明返回值为BondCalculator是受限于Python解析顺序
        assert self.bond_attributes
        assert calculator_cls
        assert self.ytm or self.clean_price or self.dirty_price
        calculator_inst = calculator_cls.new(self.calculating_date, self.bond_code, self.bond_attributes, self.ytm,
                                             self.clean_price,
                                             self.dirty_price,
                                             self.options,
                                             **kwargs)
        return calculator_inst
