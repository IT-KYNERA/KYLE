#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    int n = 3000000;
    char *sieve = (char*)calloc(n + 1, 1);
    if (!sieve) return 1;
    int count = 0;
    for (int i = 2; i <= n; i++) {
        if (!sieve[i]) {
            count++;
            for (int j = i + i; j <= n; j += i)
                sieve[j] = 1;
        }
    }
    printf("Primes: %d\n", count);
    free(sieve);
    return 0;
}
