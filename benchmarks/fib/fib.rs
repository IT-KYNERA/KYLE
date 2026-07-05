fn fib(n: i64) -> i64 {
    let mut a = 0i64;
    let mut b = 1i64;
    for _ in 0..n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    b
}

fn main() {
    println!("Result: {}", fib(10_000_000));
}
