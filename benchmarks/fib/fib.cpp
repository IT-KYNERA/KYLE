#include <stdio.h>
#include <stdint.h>

int fib(int n) {
    int a = 0, b = 1;
    for (int i = 0; i < n; i++) {
        int tmp = a + b;
        a = b;
        b = tmp;
    }
    return b;
}

int main() {
    printf("Result: %d\n", fib(500000000));
    return 0;
}
