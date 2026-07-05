fn main() {
    let mut s = String::new();
    for _ in 0..50000 {
        s = s + "x";
    }
    println!("{}", s.len());
}
