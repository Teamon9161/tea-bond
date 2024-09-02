from decimal import Decimal

from bondattr_retrieve import BondAttrRetrieval
from fi_objects import BondAttributes, CouponType, InterestType, Market
from fi_utils import pydate2str

from .wind_context import WindContext


class BondAttrRetrievalOnWind(BondAttrRetrieval):
    """使用Wind量化接口实现获取债券基本资料的接口"""

    def __init__(self, w_ctx: WindContext):
        self.w_ctx = w_ctx

    def retrieve(self, calculating_date: str, bond_code: str) -> BondAttributes:
        w_data = self.w_ctx.wind.wss(
            bond_code,
            "sec_name,carrydate,maturitydate,interesttype,couponrate,paymenttype,actualbenchmark,coupon,interestfrequency,latestpar",
            f"tradeDate={calculating_date}",
        )

        if w_data.ErrorCode != 0:
            msg = f"Invoke WindPy API error, ErrorCode={w_data.ErrorCode}"
            raise Exception(msg)

        vo = BondAttributes(bond_code=bond_code)
        vo.abbr = w_data.Data[0][0]
        vo.carry_date = pydate2str(w_data.Data[1][0].date())
        vo.maturity_date = pydate2str(w_data.Data[2][0].date())
        vo.par_value = float(w_data.Data[9][0])
        vo.cp_rate_1st = float(Decimal(str(w_data.Data[4][0])) / 100)
        vo.inst_freq = int(w_data.Data[8][0])
        vo.day_count = w_data.Data[6][0]
        inst_type_desc = w_data.Data[3][0]
        if inst_type_desc == "固定利率":
            vo.interest_type = InterestType.FIXED
        cp_type_desc = w_data.Data[7][0]
        if cp_type_desc == "附息":
            vo.cp_type = CouponType.COUPON_BEAR

        if bond_code.endswith(".IB"):
            vo.mkt = Market.IB
        elif bond_code.endswith(".SZ"):
            vo.mkt = Market.SZ
        elif bond_code.endswith(".SH"):
            vo.mkt = Market.SH
        return vo
