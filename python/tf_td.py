from datetime import date, timedelta
import re

"""
全局对象tf_tdservice是TDService的一个实例,没必要自行再实例化一个TDService
"""


class TDService:
    """可交割券有关的计算功能,当前主要是用于计算转换因子
    calc_any_cf: 根据中金所公式计算转换因子;
    cffex_tb_cf_formula: 中金所转换因子公式实现,参考http://www.cffex.com.cn/10tf/
    get_f_lasttrd_date: 计算期货合约的最后交易日
    get_f_paydate: 计算期货合约到期交割的缴款日
    """

    def get_f_lasttrd_date(self, f_symbol: str) -> date:
        """计算国债期货的最后交易日=合约到期月份的第二个星期五"""
        """根据合约代码, 依据中金所的国债期货合约最后交易日的说, 返回该合约的最后交易日"""
        # 获取年月部分
        yymm = re.sub('[A-Z]{1,}', '', f_symbol)
        yyyy = "20" + yymm[0:2]
        mm = yymm[2:]
        # 构造交割月的第一天
        begin_day_of_month = date(int(yyyy), int(mm), 1)
        # 第2个周五,月初首日的第0-6天不需要计算
        for i in range(7, 14):
            date_i = begin_day_of_month + timedelta(days=i)
            if date_i.isoweekday() == 5:
                return date_i

    def get_f_paydate(self, f_symbol) -> date:
        """获取期货合约的缴款日"""
        # 交割日为3天,其中第2天为缴款日,即最后交易日的第2个交易日,最后交易日一定为周五,所以缴款日一定是一个周二
        return self.get_f_lasttrd_date(f_symbol) + timedelta(days=4)

    @staticmethod
    def __resolve_t_type(f_symbol):
        # 获取品种代码即合约前缀字母
        return re.sub('[0-9]', '', f_symbol)

    @staticmethod
    def cffex_tb_cf_formula(n: int, c: float,
                            f: int, x: int,
                            r: float = 0.03):
        """中金所转换因子计算公式
            r：10/5/2年期国债期货合约票面利率3%；
            x：交割月到下一付息月的月份数；
            n：剩余付息次数；
            c：可交割国债的票面利率；
            f：可交割国债每年的付息次数；
            计算结果四舍五入至小数点后4位。
            """
        cf = (c / f + c / r + (1 - c / r) / pow(1 + r / f, n - 1)) / pow(1 + r / f, x * f / 12) - (
                1 - x * f / 12) * c / f
        return round(cf, 4)

    def calc_any_cf(self, f_symbol: str, b_symbol: str, b_remaining_cp_times_after_dlv: int, b_cp_rate: float,
                    b_inst_freq: int, month_number_to_next_cp_after_dlv: int,
                    fictitious_cp_rate: float = 0.03) -> float:
        """计算期货-债券的转换因子
            f_symbol: 期货合约代码
            b_symbol: 国债债券代码
            fictitious_cp_rate:虚拟券票面利率,默认值为3
            b_remaining_cp_times_after_dlv:交割券剩余付息次数,缴款日之后
            b_cp_rate:b_symbol的票面利率
            b_inst_freq:b_symbol的年付息次数
            month_number_to_next_cp_after_dlv:交割月到下个付息日之间的月份数
        """
        cf = TDService.cffex_tb_cf_formula(b_remaining_cp_times_after_dlv, b_cp_rate,
                                           b_inst_freq, month_number_to_next_cp_after_dlv, fictitious_cp_rate)
        return cf


# 全局单例
tf_tdservice = TDService()
