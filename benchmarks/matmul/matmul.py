def main():
    n = 100
    a = [1] * (n * n)
    b = [2] * (n * n)
    c = [0] * (n * n)
    for _ in range(30):
        for i in range(n):
            for j in range(n):
                s = 0
                for k in range(n):
                    s += a[i * n + k] * b[k * n + j]
                c[i * n + j] = s
    print("Matmul done:", c[n * n - 1])

main()
