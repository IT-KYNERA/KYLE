class matmul {
    public static void main(String[] args) {
        int n = 100;
        long[] a = new long[n * n];
        long[] b = new long[n * n];
        long[] c = new long[n * n];
        for (int i = 0; i < n * n; i++) { a[i] = 1; b[i] = 2; }
        for (int iter = 0; iter < 30; iter++) {
            for (int i = 0; i < n; i++) {
                for (int j = 0; j < n; j++) {
                    long sum = 0;
                    for (int k = 0; k < n; k++) {
                        sum += a[i * n + k] * b[k * n + j];
                    }
                    c[i * n + j] = sum;
                }
            }
        }
        System.out.println("Matmul done: " + c[n * n - 1]);
    }
}
