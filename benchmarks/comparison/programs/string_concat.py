def compute():
    text = ""
    for i in range(1, 5001):
        text = text + "a"
    return len(text)

print(compute())
