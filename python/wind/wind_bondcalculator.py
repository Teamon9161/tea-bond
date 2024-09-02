# -*- coding: utf-8 -*-
from fi_objects import BondAttributes
from .wind_context import WindContext
from bond_calculator import BondCalculator


class BondCalculatorOnWind(BondCalculator):

    def __init__(self, calculating_date: str, bond_code: str, bond_attributes: BondAttributes, ytm: float,
                 clean_price: float,
                 dirty_price: float,
                 options: dict,
                 **kwargs):
        super().__init__(calculating_date, bond_code, bond_attributes, ytm, clean_price, dirty_price, options)
        self.w_ctx: WindContext = kwargs['w_ctx']

    def _calc_price_with_ytm(self):
        # 1st step: 从w.wss获取全价
        calc_date = self.calculating_date.replace("-", "")
        w_data = self.w_ctx.wind.wss(self.bond_code, "calc_pv",
                                     f"balanceDate={calc_date};yield={self.ytm * 100};bondYieldType=1;bondPriceType=2")
        if w_data.ErrorCode != 0:
            raise Exception(f"Invoke WindPy API error, ErrorCode={w_data.ErrorCode}")
        self.dirty_price = w_data.Data[0][0]
        self.clean_price = self.dirty_price - self.inst_accrued

    def _calc_ytm_with_price(self):
        if self.clean_price:
            self.dirty_price = self.clean_price + self.inst_accrued
        elif self.dirty_price:
            self.clean_price = self.dirty_price - self.inst_accrued

        calc_date = self.calculating_date.replace("-", "")
        w_data = self.w_ctx.wind.wss(self.bond_code, "calc_ytm",
                                     f"balanceDate={calc_date};bondPrice={self.dirty_price};bondPriceType=2;bondYieldType=1")
        if w_data.ErrorCode != 0:
            raise Exception(f"Invoke WindPy API error, ErrorCode={w_data.ErrorCode}")
        self.ytm = w_data.Data[0][0] / 100
