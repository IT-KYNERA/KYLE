import sys
import time

def main():
    n = 1_000_000
    count = 0
    for i in range(2, n + 1):
        is_prime = True
        j = 2
        while j * j <= i:
            if i % j == 0:
                is_prime = False
                break
            j += 1
        if is_prime:
            count += 1
    print(f"Primes: {count}")

if __name__ == "__main__":
    main()
