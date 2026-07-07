fn main() {
    let n = 100usize;
    let a = vec![1i64; n * n];
    let b = vec![2i64; n * n];
    let mut c = vec![0i64; n * n];
    for _iter in 0..30 {
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0i64;
                for k in 0..n {
                    sum += a[i * n + k] * b[k * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }
    println!("Matmul done: {}", c[n * n - 1]);
}
