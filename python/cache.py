# -*- coding: utf-8 -*-
"""Cache Service and default Implementation in swhyfikit
 Copyright (C) 2022 www.swhysc.com
 author: fintech.ficc.swhysc, xueweidong@swhysc.com
 created:2022-11-19

"""

import abc
from typing import Any


class CacheService(abc.ABC):
    """缓存类接口
    本地内存缓存服务, 参见LocalMemoryCache
    基于Redis缓存服务, 待实现
    """

    @abc.abstractmethod
    def get(self, key: str) -> Any:
        return None

    def put(self, key: str, value: Any):
        self.put_ttl(key, value, -1)

    @abc.abstractmethod
    def put_ttl(self, key: str, value: Any, ttl: int):
        """ttl:time-to-live,过期时间, -1表示永不过期"""
        pass

    @abc.abstractmethod
    def __contains__(self, key):
        return False


class LocalMemoryCache(CacheService):

    def __init__(self):
        super().__init__()
        self.__cache = {}

    def get(self, key: str) -> Any:
        return self.__cache[key]

    def __contains__(self, key):
        return key in self.__cache

    def put_ttl(self, key: str, value: Any, ttl: int = -1):
        # 基于本地内存的CacheService不支持ttl, 必须传入-1
        assert ttl == -1
        self.__cache[key] = value
