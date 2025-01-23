import ctypes

from pybond import pybond

lib = ctypes.cdll.LoadLibrary(pybond.__file__)
