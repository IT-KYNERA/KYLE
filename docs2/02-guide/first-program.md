# Your First Program

Create a file named `hello.ky`:

```ky
fn main() i32:
    println("Hello, Kyle!")
    0
```

Run it:

```bash
ky run hello.ky
```

## Explanation

- `fn main() i32:` — Every program starts at `main`. It returns `i32` (exit code).
- `println("Hello, Kyle!")` — Prints to stdout.
- `0` — The last expression is the return value.

## Building

To compile without running:

```bash
ky build hello.ky
./hello
```
