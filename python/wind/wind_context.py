"""Wind based implementation for framework interface
Copyright (C) 2022 www.swhysc.com
author: fintech.ficc.swhysc
created:2022-11-19

"""

import contextlib
import traceback

from WindPy import w


class WindContext:
    """Wind量化接口对象的封装,用于w对象的生命周期管理"""

    def __init__(self):
        self.wind = w

    def init(self):
        retry_counter = 0
        while retry_counter < 2:
            if retry_counter == 0:
                print("...........................")
                print("FiKit: Connecting Wind ...")
            else:
                print("FiKit: Connect Wind failed, retry...")
            ret_obj = self.wind.start(waitTime=10)
            if ret_obj.ErrorCode == 0:
                break
            retry_counter += 1
        if not self.wind.isconnected():
            raise Exception("FiKit: Can not login Wind, Please Check Wind Terminal")
        return self

    def clear(self):
        if self.wind.isconnected():
            self.wind.stop()


@contextlib.contextmanager
def open_wind_context():
    """配合WindContext使用, 用于管理Wind量化接口客户端对象生命周期"""
    try:
        ctx = WindContext()
        ctx.init()
        print(f"FiKit: WindContext {ctx} created")
        yield ctx
    except Exception as e:
        traceback.print_exc()
    finally:
        print("...........................")
        print("FiKit: Clearing WindContext")
        if ctx:
            ctx.clear()
            print("FiKit: WindContext Cleared")
