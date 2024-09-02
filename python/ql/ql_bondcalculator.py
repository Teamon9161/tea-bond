# -*- coding: utf-8 -*-

from bond_calculator import BondCalculator
from fi_objects import BondAttributes
import QuantLib as ql
from fi_utils import strdate2ql

# 使用lamda表达式延迟实例化; 同时为传参提供基础
day_count_dict = {'A/A': lambda: ql.ActualActual(ql.ActualActual.ISMA),
                  'A/365': lambda: ql.Actual365Fixed(),
                  'A/365F': lambda: ql.Actual365Fixed(),
                  'A/360': lambda: ql.Actual360(),
                  'ACT/ACT': lambda: ql.ActualActual(ql.ActualActual.ISMA),
                  'T/365': lambda: ql.Thirty365,
                  'T/360': lambda: ql.Thirty360,
                  'BUS': lambda: ql.Business252(ql.China()),
                  'BUSIB': lambda: ql.Business252(ql.China(ql.China.IB)),
                  'BUSSSE': lambda: ql.Business252(ql.China(ql.China.SSE))
                  }


class BondCalculatorOnQL(BondCalculator):

    def __init__(self, calculating_date: str, bond_code: str, bond_attributes: BondAttributes, ytm, clean_price,
                 dirty_price, options):
        super().__init__(calculating_date, bond_code, bond_attributes, ytm, clean_price, dirty_price, options)
        self._bond_ql = None

    def _pre_calc(self):
        settlement_days = 0
        daycount_ql = day_count_dict[self.bond_attributes.day_count]()
        fixed_rate_bond = ql.FixedRateBond(settlement_days,
                                           self.bond_attributes.par_value,
                                           self._schedule_ql,
                                           [self.bond_attributes.cp_rate_1st],
                                           daycount_ql)
        self._bond_ql = fixed_rate_bond
        ql.Settings.instance().setEvaluationDate(strdate2ql(self.calculating_date))

    def _calc_price_with_ytm(self):
        self.clean_price = self._bond_ql.cleanPrice(self.ytm, self._bond_ql.dayCounter(), ql.Compounded,
                                                    self._bond_ql.frequency())
        self.dirty_price = self._bond_ql.dirtyPrice(self.ytm, self._bond_ql.dayCounter(), ql.Compounded,
                                                    self._bond_ql.frequency())
        # 覆盖应计利息
        self.inst_accrued = self.dirty_price - self.clean_price

    def _calc_ytm_with_price(self):
        if self.clean_price:
            self.dirty_price = self.clean_price + self.inst_accrued
        elif self.dirty_price:
            self.clean_price = self.dirty_price - self.inst_accrued
        self.ytm = self._bond_ql.bondYield(self.clean_price, self._bond_ql.dayCounter(), ql.Compounded,
                                           self._bond_ql.frequency())

    def _calc_duration(self):

        self.duration = ql.BondFunctions.duration(self._bond_ql, self.ytm, self._bond_ql.dayCounter(),
                                                  ql.Compounded,
                                                  self._bond_ql.frequency(),
                                                  ql.Duration.Modified,
                                                  strdate2ql(self.calculating_date))
