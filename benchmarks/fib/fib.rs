fn fib(n: i32) -> i32 {
    let mut a = 0i32;
    let mut b = 1i32;
    for _ in 0..n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    b
}

fn main() {
    println!("Result: {}", fib(500_000_000));
}
