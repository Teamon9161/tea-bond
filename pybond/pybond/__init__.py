from __future__ import annotations

import os
from pathlib import Path

from .pybond import Bond as _BondRS
from .pybond import Future
from .pybond import TfEvaluator as _TfEvaluatorRS

if os.environ.get("BONDS_INFO_PATH") is not None:
    bonds_info_environ_flag = True
    bonds_info_path = Path(os.environ.get("BONDS_INFO_PATH"))
else:
    bonds_info_environ_flag = False
    bonds_info_path = Path(__file__).parent / "data" / "bonds_info"

if not bonds_info_path.exists():
    bonds_info_path.mkdir(parents=True)


class Bond(_BondRS):
    def __new__(cls, code: str | int, path: str | None = None):
        code = str(code)
        path = bonds_info_path if path is None else path
        try:
            cls = super().__new__(cls, code, bonds_info_path)
        except ValueError as e:
            if "Read bond" in e.args[0]:
                from .download import fetch_symbols, login

                if "." not in code:
                    code = code + ".IB"
                print("Downloading bond info for ", code)
                login()
                fetch_symbols([code], save=True, save_folder=bonds_info_path)
                cls = super().__new__(cls, code, bonds_info_path)
        return cls


class TfEvaluator(_TfEvaluatorRS):
    def __new__(cls, future, bond, *args, **kwargs):
        if not isinstance(bond, Bond):
            bond = Bond(bond)
        return super().__new__(cls, future, bond, *args, **kwargs)


__all__ = ["Bond", "Future", "TfEvaluator"]
