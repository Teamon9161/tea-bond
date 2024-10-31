from __future__ import annotations

import json
from datetime import date
from decimal import Decimal
from pathlib import Path

from WindPy import w

save_folder = Path("bonds_info")
save_folder.mkdir(exist_ok=True)


def save_json(path: Path | str, data: dict) -> None:
    """
    Save data into json file in temp path.
    """
    path = Path(path)
    with path.open(mode="w+", encoding="UTF-8") as f:
        json.dump(data, f, indent=4, ensure_ascii=False)


def get_interest_type(typ: str):
    if typ == "固定利率":
        return "Fixed"
    else:
        msg = f"Unknown interest type: {typ}"
        raise ValueError(msg)


def get_payment_type(typ: str):
    if typ == "附息":
        return "Coupon_Bear"
    elif typ == "到期一次还本付息":
        return "One_Time"
    elif typ == "贴现":
        return "Zero_Coupon"
    else:
        msg = f"Unknown payment type: {typ}"
        raise ValueError(msg)


def fetch_symbols(symbols: list[str], *, save: bool = True, skip: bool = True):
    if skip:
        symbols = [s for s in symbols if not (save_folder / f"{s}.json").exists()]
    data = w.wss(
        symbols,
        "sec_name,carrydate,maturitydate,interesttype,couponrate,paymenttype,actualbenchmark,coupon,interestfrequency,latestpar",
        f"tradeDate={date.today()}",
    ).Data
    for i, symbol in enumerate(symbols):
        m = {"bond_code": symbol}
        m["mkt"] = symbol.split(".")[1].upper()
        m["abbr"] = data[0][i]  # 债券简称
        m["par_value"] = float(data[9][i])  # 面值
        m["cp_type"] = get_payment_type(data[7][i])  # 付息频率
        m["interest_type"] = get_interest_type(data[3][i])  # 付息方式
        m["cp_rate_1st"] = float(Decimal(str(data[4][i])) / 100)  # 票面利率
        m["base_rate"] = None
        m["rate_spread"] = None
        if m["cp_type"] == "Coupon_Bear":
            m["inst_freq"] = int(data[8][i])  # 年付息次数
        elif m["cp_type"] == "One_Time":
            m["inst_freq"] = 1
        elif m["cp_type"] == "Zero_Coupon":
            m["inst_freq"] = 0
        m["carry_date"] = data[1][i].strftime("%Y-%m-%d")  # 起息日
        m["maturity_date"] = data[2][i].strftime("%Y-%m-%d")  # 到期日
        m["day_count"] = data[6][i]  # 实际基准

        print(m)
        if save:
            path = save_folder / f"{symbol}.json"
            save_json(path, m)


def login():
    login_res = w.start(waitTime=8)
    if login_res.ErrorCode != 0:
        msg = f"Failed to login to Wind: {login_res.ErrorMsg}"
        raise RuntimeError(msg)


def get_all_symbols():
    sector_ids = (
        # "a101010101000000",  # 国债银行间
        # "a101010104000000",  # 政策性银行债
        "a101010201000000",  # 上交所国债
    )
    res = []
    names = []
    for sector_id in sector_ids:
        all_symbols = w.wset(
            "sectorconstituent", f"sectorid={sector_id};field=wind_code,sec_name"
        ).Data
        res.extend(all_symbols[0])
        names.extend(all_symbols[1])
    print("共有", len(res), "只债券")
    return res


if __name__ == "__main__":
    login()
    # symbols = ["220003.IB", "220021.IB", "220006.IB", "220010.IB"]
    # symbols = ["240006.IB"]

    # symbols = ["019733.SH"]
    # symbols = ["020647.SH"]
    symbols = get_all_symbols()

    fetch_symbols(symbols, save=True, skip=False)
