#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

int main() {
    int n = 100;
    int64_t *a = malloc(n * n * sizeof(int64_t));
    int64_t *b = malloc(n * n * sizeof(int64_t));
    int64_t *c = calloc(n * n, sizeof(int64_t));
    for (int i = 0; i < n * n; i++) { a[i] = 1; b[i] = 2; }
    for (int iter = 0; iter < 30; iter++) {
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                int64_t sum = 0;
                for (int k = 0; k < n; k++) {
                    sum += a[i * n + k] * b[k * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }
    printf("Matmul done: %lld\n", (long long)c[n * n - 1]);
    free(a); free(b); free(c);
    return 0;
}
