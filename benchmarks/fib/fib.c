#include <stdio.h>
#include <stdint.h>

int64_t fib(int64_t n) {
    int64_t a = 0, b = 1;
    for (int64_t i = 0; i < n; i++) {
        int64_t tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}

int main() {
    printf("Result: %lld\n", (long long)fib(10000000));
    return 0;
}
