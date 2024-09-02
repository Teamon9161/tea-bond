from cache import *
from bondattr_retrieve import *
from wind.wind_context import *
from wind.wind_bondattr import *
from pprint import pprint

if __name__ == '__main__':

    # 创建一个使用本地内存的缓存服务
    local_cache = LocalMemoryCache()

    with open_wind_context() as w_ctx:
        bondattr_retrieval = BondAttrRetrievalOnWind(w_ctx)
        ehanced_bondattr_retrieval = CachedBondAttrRetrieval(local_cache, bondattr_retrieval)

        print("---------------test begin--------------")
        calc_date = '2022-11-18'
        for bcode in ['220012.IB', '220012.IB', '220012.IB', '220012.IB']:
            battr = ehanced_bondattr_retrieval.retrieve(calculating_date=calc_date, bond_code=bcode)
            print(f">> calculating_date={calc_date}, bond_code={bcode}")
            pprint(battr.__dict__)
            print()

        print("---------------test end--------------")




