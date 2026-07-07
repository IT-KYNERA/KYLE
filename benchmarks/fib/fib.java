class fib {
    static int fibInt(int n) {
        int a = 0, b = 1;
        for (int i = 0; i < n; i++) {
            int tmp = a + b;
            a = b;
            b = tmp;
        }
        return b;
    }
    public static void main(String[] args) {
        System.out.println("Result: " + fibInt(500000000));
    }
}
