using System;

class Primes {
    static void Main() {
        int n = 3000000;
        byte[] sieve = new byte[n + 1];
        int count = 0;
        for (int i = 2; i <= n; i++) {
            if (sieve[i] == 0) {
                count++;
                for (int j = i + i; j <= n; j += i)
                    sieve[j] = 1;
            }
        }
        Console.WriteLine("Primes: " + count);
    }
}
