fn main() {
    let n = 1_000_000;
    let mut count = 0;
    for i in 2..=n {
        let mut is_prime = true;
        let mut j = 2;
        while j * j <= i {
            if i % j == 0 {
                is_prime = false;
                break;
            }
            j += 1;
        }
        if is_prime {
            count += 1;
        }
    }
    println!("Primes: {}", count);
}
