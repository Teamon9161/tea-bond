import ctypes

import llvmlite.binding

from pybond import tea_bond

lib = ctypes.cdll.LoadLibrary(tea_bond.__file__)
llvmlite.binding.load_library_permanently(tea_bond.__file__)
