def main():
    n = 3000000
    sieve = [False] * (n + 1)
    count = 0
    for i in range(2, n + 1):
        if not sieve[i]:
            count += 1
            for j in range(i + i, n + 1, i):
                sieve[j] = True
    print("Primes:", count)

main()
