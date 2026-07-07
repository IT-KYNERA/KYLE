package main

func main() {
	n := 3000000
	sieve := make([]bool, n+1)
	count := 0
	for i := 2; i <= n; i++ {
		if !sieve[i] {
			count++
			for j := i + i; j <= n; j += i {
				sieve[j] = true
			}
		}
	}
	println("Primes:", count)
}
