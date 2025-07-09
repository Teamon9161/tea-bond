import os
import ctypes
from pathlib import Path
from .evaluators import set_bond_data_path

def setup_default_bond_data_path():
    """
    设置默认的债券数据路径为 pybond/pybond/data/bonds_info
    """
    # 获取当前模块的路径
    current_dir = Path(__file__).parent.parent
    data_path = current_dir / "data" / "bonds_info"

    # 确保路径存在
    data_path.mkdir(parents=True, exist_ok=True)

    # 设置路径
    path_str = str(data_path)
    path_bytes = path_str.encode('utf-8')
    set_bond_data_path(path_bytes, len(path_bytes))

    return data_path

def set_custom_bond_data_path(path):
    """
    设置自定义的债券数据路径

    Args:
        path: 字符串路径或 Path 对象
    """
    path_str = str(path)
    path_bytes = path_str.encode('utf-8')
    set_bond_data_path(path_bytes, len(path_bytes))

# 自动设置默认路径
try:
    setup_default_bond_data_path()
except Exception as e:
    print(f"Warning: Failed to setup default bond data path: {e}")
