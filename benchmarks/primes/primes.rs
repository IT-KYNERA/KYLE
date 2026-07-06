fn main() {
    let n = 3_000_000;
    let mut sieve = vec![false; n as usize + 1];
    let mut count = 0;
    for i in 2..=n {
        if !sieve[i as usize] {
            count += 1;
            let mut j = i + i;
            while j <= n {
                sieve[j as usize] = true;
                j += i;
            }
        }
    }
    println!("Primes: {}", count);
}
