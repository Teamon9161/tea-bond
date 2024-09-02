
from fi_utils import *
from fi_objects import BondAttributes, bondattr2dict, dict2bondattr
from wind.wind_context import open_wind_context
from wind.wind_bondattr import *
from pprint import pprint

# 把从Wind获取的债券信息dump到本地文件
def dump2localfile(bond_code, local_file_path_str):
    with open_wind_context() as w_ctx:
        # 创建 bondattr_retrieval或者直接得到bondattr
        bondattr_retrieval = BondAttrRetrievalOnWind(w_ctx)
        bondattr = bondattr_retrieval.retrieve(calculating_date='2022-12-15', bond_code=bond_code)
        save_json(local_file_path_str, bondattr2dict(bondattr))


# 从本地文件反序列化到BondAttr对象
def loadlocalfile(local_file_path_str):
    d = load_json(local_file_path_str)
    return dict2bondattr(d)


if __name__ == '__main__':

    filepathstr = "C:/Users/xuewd/desktop/220012.IB.json"
    dump2localfile("220012.IB", filepathstr)
    b = loadlocalfile(filepathstr)
    pprint(b)


