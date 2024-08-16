# std/ffi/python/ffi_effects.py


def print_value(x):
    print(str(x))


def print_list(lst):
    for x in lst:
        print_value(x)
