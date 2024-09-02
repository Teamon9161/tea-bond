# -*- coding: utf-8 -*-
"""BondAttrRetrieval and Cached Decorator Implementation
 Copyright (C) 2022 www.swhysc.com
 author: fintech.ficc.swhysc, xueweidong@swhysc.com
 created:2022-11-19

"""

import abc
from fi_objects import BondAttributes
from cache import CacheService


class BondAttrRetrieval(abc.ABC):
    """Bond属性资料的获取器接口
    retrieve方法
    至少应有两个实现, 一个是可以跑在服务器上的;一个可以在本地PC终端使用Wind获取
    """

    @abc.abstractmethod
    def retrieve(self, calculating_date: str, bond_code: str) -> BondAttributes:
        return None


class CachedBondAttrRetrieval(BondAttrRetrieval):
    """使用缓存服务增强(修饰)后的Bond属性资料获取器"""
    # 为了避免cache服务中的缓存key定义冲突, 每个使用缓存的场景都应该定义自己的前缀
    CACHE_KEY_PREFIX = 'BOND_ATTR_'

    def __init__(self, cache: CacheService, retrieval: BondAttrRetrieval):
        self.delegate = retrieval
        self.cache = cache

    def retrieve(self, calculating_date: str, bond_code: str) -> BondAttributes:
        cache_key = self.CACHE_KEY_PREFIX + bond_code
        if cache_key not in self.cache:
            bond_attributes = self.delegate.retrieve(calculating_date, bond_code)
            self.cache.put(cache_key, bond_attributes)
        # 永远都从缓存中获取
        return self.cache.get(cache_key)
