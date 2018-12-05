print(''.join([a for a, b in zip(*[l for l, i in map(lambda l: (l, len([l for (a, b) in zip(*l) if a != b])), __import__('itertools').combinations(open('input'), 2)) if i == 1][0]) if a == b])[:-1])
