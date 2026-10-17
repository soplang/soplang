def compute():
    total = 0
    for i in range(1, 201):
        for j in range(1, 201):
            total = total + 1
    return total

print(compute())
