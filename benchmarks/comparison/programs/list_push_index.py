def compute():
    values = []
    for i in range(1, 5001):
        values.append(i)
    total = 0
    for i in range(0, 5000):
        total = total + values[i]
    return total

print(compute())
