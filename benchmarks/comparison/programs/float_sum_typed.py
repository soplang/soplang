def compute():
    total = 0.0
    for i in range(1, 100001):
        total = total + 0.5
    return int(total)

print(compute())
