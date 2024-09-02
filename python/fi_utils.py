import traceback
from datetime import date
from datetime import datetime, timedelta
from pathlib import Path
import json
import re
from WindPy import w
import contextlib
import QuantLib as ql
from dateutil import rrule

"""工具函数模块"""


def load_json(filepathstr: str) -> dict:
    """
    Load data from json file in temp path.
    """
    filepath: Path = Path(filepathstr)

    if filepath.exists():
        with open(filepath, mode="r", encoding="UTF-8") as f:
            data: dict = json.load(f)
        return data
    raise FileNotFoundError()


def save_json(filepathstr: str, data: dict) -> None:
    """
    Save data into json file in temp path.
    """
    filepath: Path = Path(filepathstr)
    with open(filepath, mode="w+", encoding="UTF-8") as f:
        json.dump(
            data,
            f,
            indent=4,
            ensure_ascii=False
        )


def str2pydate(datestr: str) -> date:
    return datetime.strptime(datestr, '%Y-%m-%d')


def pydate2str(pydate: date) -> str:
    return pydate.strftime('%Y-%m-%d')


def natural_days_between_dates(preceding: date, latter: date) -> int:
    """计算两个日期之间的自然日天数"""
    return (latter - preceding).days


def natural_days_between(preceding: str, latter: str) -> int:
    """计算两个日期之间的自然日天数，输入参数为%Y-%m-%d格式的日期字符串"""
    return (str2pydate(latter) - str2pydate(preceding)).days


def pydate2ql(d):
    return ql.Date(d.day, d.month, d.year)


def qldate2str(d) -> str:
    return datetime(d.year(), d.month(), d.dayOfMonth()).strftime("%Y-%m-%d")


def month_delta(from_date: str, to_date: str) -> int:
    # 开始日期的月初次日与 结束日期的月初首日之间的月份数
    start_date_s = re.sub(r"-[0-9]{2}$", "-02", from_date)
    end_date_s = re.sub(r"-[0-9]{2}$", "-01", to_date)

    start_date = datetime.strptime(start_date_s, '%Y-%m-%d')
    end_date = datetime.strptime(end_date_s, '%Y-%m-%d')

    return rrule.rrule(rrule.MONTHLY, dtstart=start_date, until=end_date).count()


def strdate2ql(d) -> ql.Date:
    if '/' in d:
        return pydate2ql(datetime.strptime(d, '%Y/%m/%d'))
    elif '-' in d:
        return pydate2ql(datetime.strptime(d, '%Y-%m-%d'))
    elif d.isdecimal() & (len(d) == 8):
        return pydate2ql(datetime.strptime(d, '%Y%m%d'))
    else:
        raise TypeError('time string format is not supported')


class FiContext:
    """
    depreciated
    """
    def __init__(self):
        self.wind = None

    def init(self):
        self.wind = w
        retry_counter = 0
        while retry_counter < 2:
            if retry_counter == 0:
                print("FiKit: Connecting Wind ...")
            else:
                print("FiKit: Connect Wind failed, retry...")
            ret_obj = w.start(waitTime=10)
            if ret_obj.ErrorCode == 0:
                break
            retry_counter += 1
        if not self.wind.isconnected():
            raise Exception("FiKit: Can not login Wind, Please Check Wind Terminal")
        return self

    def clear(self):
        if self.wind and self.wind.isconnected():
            self.wind.stop()


@contextlib.contextmanager
def fi_context():
    """
    depreciated
    """
    try:
        ctx = FiContext()
        ctx.init()
        print(f"FiKit: FiContext {ctx} created")
        yield ctx
    except Exception as e:
        traceback.print_exc()
    finally:
        print("FiKit: Clearing FiContext")
        if ctx:
            ctx.clear()
            print("FiKit: FiContext Cleared")


if __name__ == '__main__':

    print(month_delta('2022-12-12', '2023-05-21'))

    # preceding_date = '2022-09-09'
    # latter_date = '2022-09-10'
    # print(natural_days_between_dates(preceding=str2pydate(preceding_date), latter=str2pydate(latter_date)))
    # print(natural_days_between(preceding=preceding_date, latter=latter_date))
    with fi_context() as fictx:
        # ret = fictx.wind.wsd("200006.IB",
        #                       "parvalue,couponrate,interestfrequency,anal_precupn,nxcupn,accruedinterest",
        #                       "2022-09-09", "2022-09-09",
        #                       "TradingCalendar=NIB;PriceAdj=YTM")

        # wdata = fictx.wind.wset("conversionfactor", "windcode=T2212.CFE")

        calc_date = "2022-09-09".replace("-", "")
        ytm = "2.6761"
        wdata = fictx.wind.wss("200006.IB", "calc_pv",
                               f"balanceDate={calc_date};yield={ytm};bondYieldType=1;bondPriceType=2")
        if wdata.ErrorCode != 0:
            raise Exception(f"Invoke WindPy API error, ErrorCode={wdata.ErrCode}")

        print(wdata.Data[0][0])

        print(wdata)
