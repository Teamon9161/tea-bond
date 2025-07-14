from __future__ import annotations

from pathlib import Path
from .evaluators import set_bond_data_path as ffi_set_bond_data_path

def set_bond_data_path(path: Path | str):
    path_str = str(path)
    path_bytes = path_str.encode('utf-8')
    ffi_set_bond_data_path(path_bytes, len(path_bytes))
