# -*- coding: utf-8 -*-
from bond_calculator import BondCalculator
from fi_objects import InterestType, BondAttributes
import scipy.optimize as so
from fi_utils import natural_days_between_dates
from .formulas import *
import datetime as dt


class PrimitiveBondCalculator(BondCalculator):

    def __init__(self, calculating_date: str, bond_code: str, bond_attributes: BondAttributes, ytm, clean_price,
                 dirty_price, options):
        super().__init__(calculating_date, bond_code, bond_attributes, ytm, clean_price, dirty_price, options)

    def _pre_calc(self):
        pass

    def _map_ib_formula_i(self):
        """将本对象中的各字段映射到公式中的输入变量名"""
        PV = self.dirty_price
        f = self.bond_attributes.inst_freq
        C = self.bond_attributes.cp_rate_1st * self.bond_attributes.par_value
        M = self.bond_attributes.par_value
        d = self.remaining_days2nextcp
        n = self.remaining_cp_num
        D = self.remaining_days2mature
        y = self.ytm
        TS = self.present_cp_period_days

        return PV, f, C, M, d, n, y, TS, D

    def __TY(self):
        """公式4中的TY,最后一个计息年度的天数"""
        final_date_ql = self._schedule_ql.endDate()
        for i in range(len(self._schedule_ql) - 2, 0, -1):
            # 从后向前,获取离到期日最近且跨年的付息日
            if self._schedule_ql[i].year() != final_date_ql.year():
                candidate = natural_days_between_dates(self._schedule_ql[i].to_date(), final_date_ql.to_date())
                if candidate < 360:
                    # <360说明是一年多次付息的情况,排除该付息日继续向前找
                    continue
                else:
                    return candidate

    def _calc_price_with_ytm(self):
        PV, f, C, M, d, n, y, TS, D = self._map_ib_formula_i()
        if self.remaining_cp_num > 1:
            self.dirty_price = ib_f6(M, C, f, y, n, TS, d)
            # 覆盖应计利息
            self.clean_price = self.dirty_price - self.inst_accrued
        else:
            FV = M + C / f
            TY = self.__TY()
            self.dirty_price = ib_f5(y, FV, D, TY)
            # 覆盖应计利息
            self.clean_price = self.dirty_price - self.inst_accrued

    def _calc_ytm_with_price(self):
        if self.clean_price:
            self.dirty_price = self.clean_price + self.inst_accrued
        elif self.dirty_price:
            self.clean_price = self.dirty_price - self.inst_accrued

        PV, f, C, M, d, n, ignored, TS, D = self._map_ib_formula_i()

        if self.bond_attributes.interest_type == InterestType.FIXED:
            if self.remaining_cp_num > 1:
                """不在最后一个付息周期内使用公式6"""
                # 基于最小二乘法
                self.ytm = so.fsolve(lambda y: ib_f6(M, C, f, y, n, TS, d) - PV, 0.01)[0]
            else:
                """在最后一个付息周期内使用公式4"""
                FV = M + C / f
                TY = self.__TY()
                self.ytm = ib_f4(PV, FV, D, TY)

        else:
            # todo: 后面再支持,这块写出来以后用策略模式整体重新实现
            raise Exception(f"Not supported bond interest type:{self.bond_attributes.interest_type}")

    def _calc_duration(self):
        PV, f, C, M, d, n, y, TS, D = self._map_ib_formula_i()
        self.duration = ib_f6_D(M, C, f, y, n, TS, d)

