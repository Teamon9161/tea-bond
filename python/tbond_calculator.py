import datetime

from fi_utils import natural_days_between, str2pydate, pydate2str, fi_context, FiContext, qldate2str
import pprint
import QuantLib as ql

"""T-Bond Calculator
author: xueweidong@swhysc.com
created:2022-10-29

depreciated 该文件已作废,改为使用bond_calculator.py
"""


class TBondCalculator:
    tb_wind_data_cache = {}

    def __init__(self, b_symbol: str, calculating_date: str, ytm: float):
        self.b_symbol: str = b_symbol
        self.ytm: float = ytm  # 注意单位, 0.023456代表 2.3456%
        self.calculating_date: str = calculating_date

        # 以下字段从wind.wsd获取
        self.par_value: float = 100.00  # 面值
        self.coupon_rate: float = None  # 票面利率,注意单位, 0.023456代表 2.3456%
        self.inst_freq: int = None  # 付息频率
        # self.inst_day_count = None  # 计息基准
        self.coupon: float = None  # 单次付息金额, par_value*coupon_rate/inst_freq
        self.pre_cp_date: str = None  # 计算日前一付息日
        self.next_cp_date: str = None  # 计算日下一付息日
        self.carry_date: str = None  # 起息日
        self.maturity_date: str = None  # 到期日
        self.inst_accrued: float = None  # 应计利息, 不使用Wind的结果,
        # 自己计算应计利息：inst_accrued=coupon*inst_accrued_days/inst_current_period_days

        self._schedule_ql: ql.Schedule = None
        # 以下字段自己计算
        self.inst_current_period_days: int = None  # 当前付息周期实际天数
        self.inst_accrued_days: int = None  # 前一付息日到计算日之间的天数
        self.inst_next_coupon_days: int = None  # 计算日到下一付息日的天数,Wind的结果不对,Wind给的是当前日期到下一付息日的天数
        # 使用Wind债券计算器得到dirty_price
        self.net_price: float = None
        self.dirty_price: float = None

    def __enrich_from_wind(self, ctx: FiContext):
        if self.b_symbol not in TBondCalculator.tb_wind_data_cache:
            w_data = ctx.wind.wsd(self.b_symbol,
                                  "parvalue,couponrate,interestfrequency,carrydate,maturitydate",
                                  self.calculating_date,
                                  self.calculating_date,
                                  "N=0;credibility=1")
            if w_data.ErrorCode != 0:
                raise Exception(f"Invoke WindPy API error, ErrorCode={w_data.ErrCode}")
            # 写入缓存
            TBondCalculator.tb_wind_data_cache[self.b_symbol] = {'par_value': float(w_data.Data[0][0].split(" ")[0]),
                                                                 'coupon_rate': float(w_data.Data[1][0]),
                                                                 'inst_freq': int(w_data.Data[2][0]),
                                                                 'carry_date': pydate2str(w_data.Data[3][0].date()),
                                                                 'maturity_date': pydate2str(w_data.Data[4][0].date())}

        cache_rec = TBondCalculator.tb_wind_data_cache[self.b_symbol]
        self.par_value = cache_rec['par_value']
        self.coupon_rate = cache_rec['coupon_rate']/100  # 存到缓存里的是Wind的原始数据
        self.inst_freq = cache_rec['inst_freq']
        self.carry_date = cache_rec['carry_date']
        self.maturity_date = cache_rec['maturity_date']
        self.coupon = self.par_value * self.coupon_rate / self.inst_freq

    def __dirty_price_by_wind(self, ctx: FiContext):
        calc_date = self.calculating_date.replace("-", "")
        # ytm, Wind输入的是%单位的
        w_data = ctx.wind.wss(self.b_symbol, "calc_pv",
                              f"balanceDate={calc_date};yield={self.ytm*100};bondYieldType=1;bondPriceType=2")
        if w_data.ErrorCode != 0:
            raise Exception(f"Invoke WindPy API error, ErrorCode={w_data.ErrCode}")
        self.dirty_price = w_data.Data[0][0]

    def calc(self, ctx: FiContext):
        self.__enrich_from_wind(ctx)

        self.pre_cp_date, self.next_cp_date = self.obtain_cp_dates_section(self.calculating_date)

        self.inst_current_period_days = natural_days_between(self.pre_cp_date, self.next_cp_date)
        self.inst_accrued_days = natural_days_between(self.pre_cp_date, self.calculating_date)
        self.inst_next_coupon_days = natural_days_between(self.calculating_date, self.next_cp_date)
        self.inst_accrued = self.coupon * self.inst_accrued_days / self.inst_current_period_days

        self.__dirty_price_by_wind(ctx)
        self.net_price = self.dirty_price - self.inst_accrued

        return self

    def _ensure_qlschedule(self):
        """使用 QuantLib 构造一个该债券的现金流Schedule"""
        if not self._schedule_ql:
            carryDate_py = str2pydate(self.carry_date)
            maturityDate_py = str2pydate(self.maturity_date)
            issueDate_ql = ql.Date(carryDate_py.day, carryDate_py.month, carryDate_py.year)  # 起息日
            maturityDate_ql = ql.Date(maturityDate_py.day, maturityDate_py.month, maturityDate_py.year)  # 到期日
            tenor_ql = ql.Period(ql.Semiannual) if self.inst_freq == 2 else ql.Period(ql.Annual)  # 付息期限
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


if __name__ == "__main__":

    with fi_context() as fi_ctx:
        tb1_calc = TBondCalculator(b_symbol="200006.IB", calculating_date="2022-09-09", ytm=0.026761).calc(fi_ctx)
        pprint.pprint(tb1_calc.__dict__)

        tb2_calc = TBondCalculator(b_symbol="200006.IB", calculating_date="2022-09-09", ytm=0.026762).calc(fi_ctx)
        pprint.pprint(tb2_calc.__dict__)

