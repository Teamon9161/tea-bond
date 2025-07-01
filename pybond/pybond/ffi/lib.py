import ctypes

import llvmlite.binding

from pybond import pybond

lib = ctypes.cdll.LoadLibrary(pybond.__file__)
llvmlite.binding.load_library_permanently(pybond.__file__)
