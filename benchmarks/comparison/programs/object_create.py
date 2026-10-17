def compute():
    total = 0
    for i in range(0, 5000):
        point = {"x": i, "y": i * 2}
        total = total + point["x"] + point["y"]
    return total

print(compute())
