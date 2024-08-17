# std/ffi/python/ffi_effects.py


def print_value(x):
    print(x)


def print_list(lst):
    for x in lst:
        print_value(x)
