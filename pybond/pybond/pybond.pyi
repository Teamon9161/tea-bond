from datetime import date
from pathlib import Path

class Bond:
    """A class representing a bond instrument."""

    def __init__(self, code: str | int, path: str | Path | None = None) -> None:
        """
        Create a new Bond instance.

        Args:
            code: Bond code
            path: Path to directory containing bond data. Defaults to None

        Raises:
            ValueError: If bond data cannot be read or parsed
        """

    def copy(self) -> Bond: ...
    @property
    def code(self) -> str:
        """债券代码, 不包含交易所后缀"""

    @property
    def full_code(self) -> str:
        """债券代码, 包含交易所后缀"""

    @property
    def market(self) -> str:
        """债券市场"""

    @property
    def abbr(self) -> str:
        """债券简称"""

    @property
    def name(self) -> str:
        """债券名称 (alias for abbr)"""

    @property
    def par_value(self) -> float:
        """债券面值"""

    @property
    def coupon_type(self) -> str:
        """息票品种"""

    @property
    def interest_type(self) -> str:
        """息票利率类型"""

    @property
    def coupon_rate(self) -> float:
        """票面利率, 浮动付息债券仅表示发行时票面利率"""

    @property
    def cp_rate(self) -> float:
        """票面利率, 浮动付息债券仅表示发行时票面利率"""

    @property
    def base_rate(self) -> float | None:
        """基准利率, 浮动付息债券适用"""

    @property
    def rate_spread(self) -> float | None:
        """固定利差, 浮动付息债券适用"""

    @property
    def inst_freq(self) -> int:
        """年付息次数"""

    @property
    def carry_date(self) -> date:
        """起息日"""

    @property
    def maturity_date(self) -> date:
        """到期日"""

    @property
    def day_count(self) -> str:
        """计息基准"""

    def is_zero_coupon(self) -> bool:
        """是否为零息债券"""

    def remain_year(self, date: date | None = None) -> float:
        """剩余年数"""

    def issue_year(self) -> int:
        """发行年数"""

    @property
    def get_coupon(self) -> float:
        """
        获取区间付息(单个付息周期的利息金额)

        区间付息 = 票面利率 * 面值 / 年付息次数
        """

    @property
    def last_cp_year_days(self) -> int:
        """最后一个计息年度的天数"""

    def nearest_cp_date(self, date: date) -> tuple[date, date]:
        """获取上一付息日和下一付息日"""

    def remain_cp_num(self, date: date, next_cp_date: date | None = None) -> int:
        """剩余的付息次数"""

    def remain_cp_num_until(
        self, date: date, until_date: date, next_cp_date: date | None = None
    ) -> int:
        """剩余的付息次数直到指定日期"""

    def remain_cp_dates_until(
        self, date: date, until_date: date, next_cp_date: date | None = None
    ) -> list[date]:
        """剩余的付息日期列表(不包含指定日期)"""

    def calc_ytm_with_price(
        self,
        dirty_price: float,
        date: date,
        cp_dates: tuple[date, date] | None = None,
        remain_cp_num: int | None = None,
    ) -> float:
        """通过债券全价计算ytm"""

class Future:
    """A class representing a treasury futures contract."""

    def __init__(self, code: str) -> None:
        """
        Create a new Future instance.

        Args:
            code: Future contract code
        """

    def copy(self) -> Future: ...
    def is_deliverable(
        self,
        carry_date: date,
        maturity_date: date,
        delivery_date: date | None = None,
    ) -> bool:
        """
        判断是否是可交割券

        Args:
            carry_date: 起息日
            maturity_date: 到期日
            delivery_date: 可以传入已计算过的期货配对缴款日避免重复计算
        """

    def last_trading_date(self) -> date:
        """
        计算期货合约的最后交易日

        计算国债期货的最后交易日=合约到期月份的第二个星期五
        根据合约代码, 依据中金所的国债期货合约最后交易日的说, 返回该合约的最后交易日
        获取年月部分
        """

    def deliver_date(self) -> date:
        """
        获取期货合约的配对缴款日

        交割日为3天,其中第2天为缴款日,即最后交易日的第2个交易日,最后交易日一定为周五,所以缴款日一定是一个周二
        """

    def future_type(self) -> str:
        """获取期货合约的类型"""

class TfEvaluator:
    """A class for evaluating treasury futures."""

    def __init__(
        self,
        future: Future | str,
        bond: Bond | str | int,
        date: date | str | None = None,
        future_price: float = float("nan"),
        bond_ytm: float = float("nan"),
        capital_rate: float = float("nan"),
        reinvest_rate: float | None = None,
    ) -> None: ...
    def copy(self) -> TfEvaluator: ...

    def date(self) -> date:
        """获取计算日期"""

    @property
    def bond_code(self) -> str:
        """获取债券代码"""

    @property
    def bond_ytm(self) -> float:
        """获取债券收益率"""

    @property
    def future(self) -> str:
        """获取期货代码"""

    @property
    def future_price(self) -> float:
        """获取期货价格"""

    @property
    def deliverable(self) -> bool:
        """判断债券是否是期货的可交割券"""

    def with_deliver_date(self) -> TfEvaluator:
        """计算期货配对缴款日"""

    def with_nearest_cp_dates(self) -> TfEvaluator:
        """计算前一付息日和下一付息日"""

    def with_deliver_cp_dates(self) -> TfEvaluator:
        """计算交割日的前一付息日和下一付息日"""

    def with_remain_days_to_deliver(self) -> TfEvaluator:
        """计算到交割的剩余天数"""

    def with_remain_cp_num(self) -> TfEvaluator:
        """计算剩余付息次数"""

    def with_accrued_interest(self) -> TfEvaluator:
        """计算应计利息"""

    def with_dirty_price(self) -> TfEvaluator:
        """计算债券全价"""

    def with_clean_price(self) -> TfEvaluator:
        """计算债券净价"""

    def with_duration(self) -> TfEvaluator:
        """计算修正久期"""

    def with_cf(self) -> TfEvaluator:
        """计算转换因子"""

    def with_basis_spread(self) -> TfEvaluator:
        """计算基差"""

    def with_deliver_accrued_interest(self) -> TfEvaluator:
        """
        计算国债期货交割应计利息

        国债期货交割应计利息=区间付息* (国债期货交割缴款日 - 国债期货交割前一付息日) / (国债期货交割下一付息日 - 国债期货交割前一付息日)

        按中金所发布公式, 计算结果四舍五入至小数点后7位
        """

    def with_future_dirty_price(self) -> TfEvaluator:
        """计算期货全价(发票价格)"""

    def with_remain_cp_to_deliver(self) -> TfEvaluator:
        """计算期间付息"""

    def with_deliver_cost(self) -> TfEvaluator:
        """
        计算交割成本

        交割成本=债券全价-期间��息
        """

    def with_f_b_spread(self) -> TfEvaluator:
        """计算期现价差"""

    def with_carry(self) -> TfEvaluator:
        """
        计算持有收益

        持有收益 = (交割日应计-交易日应计 + 期间付息) + 资金成本率*(加权平均期间付息-债券全价*剩余天数/365)
        """

    def with_net_basis_spread(self) -> TfEvaluator:
        """
        计算净基差

        净基差=基差-持有收益
        """

    def with_irr(self) -> TfEvaluator:
        """计算内部收益率IRR"""

    def with_future_ytm(self) -> TfEvaluator:
        """计算期货隐含收益率"""

    def calc_all(self) -> TfEvaluator:
        """计算所有指标"""

    def update(
        self,
        future_price: float | None = None,
        bond_ytm: float | None = None,
        date: date | None = None,
        future: Future | str | None = None,
        bond: Bond | str | int | None = None,
        capital_rate: float | None = None,
        reinvest_rate: float | None = None,
    ) -> TfEvaluator:
        """
        根据新的日期、债券和期货信息更新评估器

        此函数会根据输入的新信息更新评估器的各个字段,
        并根据变化情况决定是否保留原有的计算结果。
        当输入的参数为None时, 会沿用原始评估器中的值。
        """

    @property
    def accrued_interest(self) -> float:
        """应计利息, 如果未计算会自动计算后返回"""

    @property
    def deliver_accrued_interest(self) -> float:
        """国债期货交割应计利息, 如果未计算会自动计算后返回"""

    @property
    def cf(self) -> float:
        """转换因子, 如果未计算会自动计算后返回"""

    @property
    def dirty_price(self) -> float:
        """债券全价, 如果未计算会自动计算后返回"""

    @property
    def clean_price(self) -> float:
        """债券净价, 如果未计算会自动计算后返回"""

    @property
    def future_dirty_price(self) -> float:
        """期货全价, 如果未计算会自动计算后返回"""

    @property
    def deliver_cost(self) -> float:
        """交割成本, 如果未计算会自动计算后返回"""

    @property
    def basis_spread(self) -> float:
        """基差, 如果未计算会自动计算后返回"""

    @property
    def f_b_spread(self) -> float:
        """期现价差, 如果未计算会自动计算后返回"""

    @property
    def carry(self) -> float:
        """持有收益, 如果未计算会自动计算后返回"""

    @property
    def net_basis_spread(self) -> float:
        """净基差, 如果未计算会自动计算后返回"""

    @property
    def duration(self) -> float:
        """修正久期, 如果未计算会自动计算后返回"""

    @property
    def irr(self) -> float:
        """内部收益率, 如果未计算会自动计算后返回"""

    @property
    def deliver_date(self) -> date:
        """期货配对缴款日, 如果未计算会自动计算后返回"""

    @property
    def cp_dates(self) -> tuple[date, date]:
        """前一付息日和下一付息日, 如果未计算会自动计算后返回"""

    @property
    def deliver_cp_dates(self) -> tuple[date, date]:
        """期货交割日的前一付息日和下一付息日, 如果未计算会自动计算后返回"""

    @property
    def remain_cp_num(self) -> int:
        """债券剩余付息次数, 如果未计算会自动计算后返回"""

    @property
    def remain_cp_to_deliver(self) -> float:
        """到交割的期间付息, 如果未计算会自动计算后返回"""

    @property
    def remain_cp_to_deliver_wm(self) -> float:
        """加权平均到交割的期间付息, 如果未计算会自动计算后返回"""

    @property
    def future_ytm(self) -> float:
        """期货隐含收益率, 如果未计算会自动计算后返回"""
