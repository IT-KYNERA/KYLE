using System;

class Fib {
    static void Main() {
        unchecked {
            long a = 0, b = 1;
            for (int i = 0; i < 10000000; i++) {
                long tmp = a + b;
                a = b;
                b = tmp;
            }
            Console.WriteLine(b);
        }
    }
}
