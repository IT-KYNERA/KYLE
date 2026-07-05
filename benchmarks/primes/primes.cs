using System;

class Primes {
    static void Main() {
        int n = 1_000_000;
        int count = 0;
        for (int i = 2; i <= n; i++) {
            bool isPrime = true;
            for (int j = 2; j * j <= i; j++) {
                if (i % j == 0) {
                    isPrime = false;
                    break;
                }
            }
            if (isPrime) count++;
        }
        Console.WriteLine("Primes: " + count);
    }
}
