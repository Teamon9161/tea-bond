from prim.formulas import *
from fi_utils import natural_days_between

if __name__ == '__main__':
    # for 220012.IB at 2022-11-18
    M, C, f, y, n, TS, d = 100, 2.75, 1, 2.7900/100, 7, 365, natural_days_between('2022-11-18', '2023-06-15')
    D = ib_f6_D(M, C, f, y, n, TS, d)
    print(D)