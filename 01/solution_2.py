print((lambda s=set(): next(i for i, st, _ in ([i, s.copy(), s.add(i)] for i in __import__('itertools').accumulate(__import__('itertools').cycle(map(int, open('input'))))) if i in st))())
