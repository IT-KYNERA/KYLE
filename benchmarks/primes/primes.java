public class primes {
    public static void main(String[] args) {
        int n = 1_000_000;
        int count = 0;
        for (int i = 2; i <= n; i++) {
            boolean isPrime = true;
            for (int j = 2; j * j <= i; j++) {
                if (i % j == 0) {
                    isPrime = false;
                    break;
                }
            }
            if (isPrime) count++;
        }
        System.out.println("Primes: " + count);
    }
}
