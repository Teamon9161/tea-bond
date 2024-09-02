# -*- coding: utf-8 -*-

import numpy as np

""" 集中定义银行间与交易所市场各自的计算公式

    银行间市场的公式变量, 参见:全国银行间债券市场债券到期收益率计算标准调整对照表2007
    y ：到期收益率
    FV ：到期兑付日债券本息和
    PV ：债券全价
    D ：债券结算日至到期兑付日的实际天数
    M ：债券面值
    N ：债券期限（年）, 即从起息日至到期兑付日的整年数
    C ：债券票面年利息
    f ：年付息频率
    TY: 当前计息年度的实际天数, 算头不算尾
    d ：债券结算日至下一最近付息日之间的实际天数
    n ：结算日至到期兑付日的债券付息次数
    M ：债券面值
    TS: 当前付息周期的实际天数
"""


def ib_f6(M, C, f, y, n, TS, d):
    cp_list = [(C / f) / pow(1 + y / f,
                             d / TS + i) for i in np.arange(0, n)]

    return np.sum(cp_list) + M / pow(1 + y / f,
                                     d / TS + n - 1)


def ib_f6_D(M, C, f, y, n, TS, d):
    """计算久期Duration, 简写为D"""
    # 利息-时间
    cashflow = [((C / f) / pow(1 + y / f,
                               d / TS + i), (d / 365 + i / f)) for i in np.arange(0, n)]
    # 本金-时间
    cashflow.append((M / pow(1 + y / f,
                             d / TS + n - 1), d / 365 + (n - 1) / f))
    PV = np.sum([c for c, t in cashflow])
    # 麦考利久期
    D_M = np.sum([t * c / PV for c, t in cashflow])
    D = D_M / (1 + y / f)
    return D


def ib_f4(PV, FV, D, TY):
    return (FV - PV) / PV / (D / TY)

def ib_f5(y, FV, D, TY):
    return FV / (y * D / TY + 1)
