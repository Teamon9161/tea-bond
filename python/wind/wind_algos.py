# -*- coding: utf-8 -*-
from .wind_context import WindContext
from tf_evaluator import FYtmAlgo


class WindFYtmAlgo(FYtmAlgo):

    def __init__(self, w_ctx: WindContext):
        self.w_ctx = w_ctx

    def __call__(self, *args, **kwargs):
        b_symbol = self.tf_evaluator.b_symbol
        clean_price = self.tf_evaluator.f_px * self.tf_evaluator.cf
        calc_date = self.tf_evaluator.deliver_date.replace("-", "")
        w_data = self.w_ctx.wind.wss(b_symbol, "calc_ytm",
                                     f"balanceDate={calc_date};bondPrice={clean_price};bondPriceType=1;bondYieldType=1")
        if w_data.ErrorCode != 0:
            raise Exception(f"Invoke WindPy API error, ErrorCode={w_data.ErrorCode}")
        self.tf_evaluator.f_ytm = w_data.Data[0][0] / 100
