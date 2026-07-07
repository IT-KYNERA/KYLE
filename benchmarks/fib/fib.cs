using System;

class Fib {
    static int FibInt(int n) {
        int a = 0, b = 1;
        for (int i = 0; i < n; i++) {
            int tmp = a + b;
            a = b;
            b = tmp;
        }
        return b;
    }
    static void Main() {
        Console.WriteLine("Result: " + FibInt(500000000));
    }
}
