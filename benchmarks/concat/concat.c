#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    char *s = strdup("");
    size_t len = 0;
    for (int i = 0; i < 500000; i++) {
        len++;
        s = realloc(s, len + 1);
        s[len - 1] = 'x';
        s[len] = '\0';
    }
    printf("%zu\n", len);
    free(s);
    return 0;
}
