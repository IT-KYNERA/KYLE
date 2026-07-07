class primes {
    public static void main(String[] args) {
        int n = 3000000;
        boolean[] sieve = new boolean[n + 1];
        int count = 0;
        for (int i = 2; i <= n; i++) {
            if (!sieve[i]) {
                count++;
                for (int j = i + i; j <= n; j += i)
                    sieve[j] = true;
            }
        }
        System.out.println("Primes: " + count);
    }
}
