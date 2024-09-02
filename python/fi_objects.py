# -*- coding: utf-8 -*-
"""领域对象的字典
 Copyright (C) 2022 www.swhysc.com
 author: fintech.ficc.swhysc
 created:2022-11-19

"""
from enum import Enum
from dataclasses import dataclass, asdict


class CouponType(Enum):
    """息票类型
    CouponBear: 附息债券
    Zero_Coupon: 零息债券
    One_Time
    """
    COUPON_BEAR = 'Coupon_Bear'
    ZERO_COUPON = 'Zero_Coupon'
    ONE_TIME = 'One_Time'


class InterestType(Enum):
    """息票利率类型
    Fixed: 固定利率
    Floating: 浮动利率
    Progressive: 累进利率
    Zero: 零息
    """
    FIXED = 'Fixed'
    FLOATING = 'Floating'
    PROGRESSIVE: 'Progressive'
    ZERO = 'Zero'


class Market(Enum):
    """债券市场
    SSE: 上交所, SH是其同义词
    SZE: 深交所, SZ是其同义词
    IB: 银行间
    """
    IB = 'IB'
    SSE = 'SH'
    SH = 'SH'
    SZE = 'SZ'
    SZ = 'SZ'


@dataclass
class BondAttributes:
    """债券的基本属性"""
    bond_code: str  # 债券代码
    mkt: Market = None  # 市场
    abbr: str = None  # 债券简称
    par_value: float = None  # 债券面值
    cp_type: CouponType = None  # 息票品种
    interest_type: InterestType = None  # 息票利率类型
    cp_rate_1st: float = None  # 票面利率, 浮动付息债券仅表示发行时票面利率
    base_rate: float = None  # 基准利率, 浮动付息债券适用
    rate_spread: float = None  # 固定利差, 浮动付息债券适用
    inst_freq: int = None  # 年付息次数
    carry_date: str = None  # 起息日
    maturity_date: str = None  # 到期日
    day_count: str = None  # 计息基准, 如A/365F


def enum2str(enumobj: Enum) -> str:
    return enumobj.value


def str2enum(EnumType, value: str) -> Enum:
    return EnumType(value)


def bondattr2dict(bondattr: BondAttributes) -> dict:
    d = asdict(bondattr)
    d["mkt"] = enum2str(d["mkt"])
    d["cp_type"] = enum2str(d["cp_type"])
    d["interest_type"] = enum2str(d["interest_type"])
    return d


def dict2bondattr(d: dict) -> BondAttributes:
    if "mkt" in d:
        d["mkt"] = str2enum(Market, d["mkt"])
    if "cp_type" in d:
        d["cp_type"] = str2enum(CouponType, d["cp_type"])
    if "interest_type" in d:
        d["interest_type"] = str2enum(InterestType, d["interest_type"])
    b = BondAttributes(**d)

    return b
