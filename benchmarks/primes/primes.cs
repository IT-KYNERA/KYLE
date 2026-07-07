using System;

class Primes {
    static void Main() {
        int n = 3000000;
        bool[] sieve = new bool[n + 1];
        int count = 0;
        for (int i = 2; i <= n; i++) {
            if (!sieve[i]) {
                count++;
                for (int j = i + i; j <= n; j += i)
                    sieve[j] = true;
            }
        }
        Console.WriteLine("Primes: " + count);
    }
}
