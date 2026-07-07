def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return b

print("Result:", fib(5000000))
