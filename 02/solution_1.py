print(reduce(lambda x, y: x * y, map(lambda i: len([i for counts in [[len(list(values))for key, values in l] for l in map(__import__('itertools').groupby, map(sorted, open('input')))] if i in counts]), [2, 3])))
