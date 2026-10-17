def add_one(value):
    return value + 1

def compute():
    total = 0
    for i in range(1, 20001):
        total = add_one(total)
    return total

print(compute())
