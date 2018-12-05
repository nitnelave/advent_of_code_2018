import itertools
def find_repeated_frequency():
    sum = 0
    seen = set()
    for n in itertools.cycle(map(int, open('input'))):
        if sum in seen:
            return sum
        seen.add(sum)
        sum += n
print(find_repeated_frequency())
