import sys

def main():
    n = 3_000_000
    sieve = bytearray(n + 1)
    count = 0
    for i in range(2, n + 1):
        if sieve[i] == 0:
            count += 1
            step = i
            for j in range(i + i, n + 1, step):
                sieve[j] = 1
    print(f"Primes: {count}")

if __name__ == "__main__":
    main()
