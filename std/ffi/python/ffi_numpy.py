# std/ffi/python/ffi_numpy.py

import numpy as np


def array(data):
    return np.array(data)


def mean(arr):
    return np.mean(arr)


def std(arr):
    return np.std(arr)
