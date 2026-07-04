#!/bin/bash
set -eu

KY_SRC="$(cd "$(dirname "$0")/.." && pwd)"
KY="$KY_SRC/target/release/ky"
RESULTS="$KY_SRC/BENCHMARK.md"
TMPDIR="/tmp/ky_bench_$$"
mkdir -p "$TMPDIR"
cd "$TMPDIR"

echo "# Kyle Benchmark Results" > "$RESULTS"
echo "" >> "$RESULTS"
echo "Date: $(date)" >> "$RESULTS"
echo "Machine: $(uname -a)" >> "$RESULTS"
echo "" >> "$RESULTS"

bench() {
    local binary=$1
    shift
    if [ ! -x "$binary" ]; then echo "BIN_NOT_FOUND|"; return; fi
    local start_ns=$(python3 -c 'import time; print(time.time_ns())')
    local output=$($binary "$@" 2>/dev/null || echo "ERR")
    local end_ns=$(python3 -c 'import time; print(time.time_ns())')
    local elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
    echo "${elapsed_ms}ms|$output"
}

build_ky() {
    local src=$1
    local dir=$(dirname "$src")
    local base=$(basename "$src" .ky)
    (cd "$dir" && "$KY" build "$src" >/dev/null 2>&1) || true
    echo "$dir/target/debug/$base"
}

build_rs() {
    local src=$1
    local base=$(basename "$src" .rs)
    rustc -O "$src" -o "$TMPDIR/${base}_rs" 2>/dev/null
    echo "$TMPDIR/${base}_rs"
}

# ═══════════════════════════════════════════════════
echo "## 1. Primes (3M)" >> "$RESULTS"
echo "### Compilation" >> "$RESULTS"
echo "| Metric | Kyle | Rust |" >> "$RESULTS"
echo "|--------|------|------|" >> "$RESULTS"

cp "$KY_SRC/examples/bench/primes.ky" "$TMPDIR/."
cp "$KY_SRC/examples/bench/primes.rs" "$TMPDIR/."
KY_P=$(build_ky "$TMPDIR/primes.ky")
RS_P=$(build_rs "$TMPDIR/primes.rs")

KY_SIZE=$(stat -f%z "$KY_P" 2>/dev/null || echo "0")
RS_SIZE=$(stat -f%z "$RS_P" 2>/dev/null || echo "0")
echo "| Binary size | ${KY_SIZE} | ${RS_SIZE} |" >> "$RESULTS"

strip "$KY_P" 2>/dev/null || true; strip "$RS_P" 2>/dev/null || true
echo "| Stripped size | $(stat -f%z "$KY_P" 2>/dev/null) | $(stat -f%z "$RS_P" 2>/dev/null) |" >> "$RESULTS"

echo "" >> "$RESULTS"
echo "### Execution" >> "$RESULTS"
echo "| Lang | Time | Result |" >> "$RESULTS"
echo "|------|------|--------|" >> "$RESULTS"
echo "| Kyle | $(bench "$KY_P" 3000000) |" >> "$RESULTS"
echo "| Rust | $(bench "$RS_P" 3000000) |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "## 2. Fibonacci 40" >> "$RESULTS"
echo "| Lang | Time | Result |" >> "$RESULTS"
echo "|------|------|--------|" >> "$RESULTS"

cat > "$TMPDIR/fib.ky" << 'KYEOF'
fn fib(n: i32) i32:
    if n <= 1:
        n
    else:
        fib(n - 1) + fib(n - 2)

fn main() i32:
    r = fib(40)
    print(r)
    print("\n")
    0
KYEOF
cat > "$TMPDIR/fib.rs" << 'EOF'
fn fib(n: i32) -> i32 { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }
fn main() { println!("{}", fib(40)); }
EOF

KY_F=$(build_ky "$TMPDIR/fib.ky")
RS_F=$(build_rs "$TMPDIR/fib.rs")
echo "| Kyle | $(bench "$KY_F") |" >> "$RESULTS"
echo "| Rust | $(bench "$RS_F") |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "## 3. String concat (100k)" >> "$RESULTS"
echo "| Lang | Time | Result |" >> "$RESULTS"
echo "|------|------|--------|" >> "$RESULTS"

cat > "$TMPDIR/strcat.ky" << 'KYEOF'
fn main() i32:
    s = ""
    i: &i32 = 0
    while i < 100000:
        s = s + "x"
        i = i + 1
    print(len(s))
    print("\n")
    0
KYEOF
cat > "$TMPDIR/strcat.rs" << 'EOF'
fn main() { let mut s = String::new(); for _ in 0..100000 { s.push('x'); } println!("{}", s.len()); }
EOF

KY_S=$(build_ky "$TMPDIR/strcat.ky")
RS_S=$(build_rs "$TMPDIR/strcat.rs")
echo "| Kyle | $(bench "$KY_S") |" >> "$RESULTS"
echo "| Rust | $(bench "$RS_S") |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "## 4. List push (100k)" >> "$RESULTS"
echo "| Lang | Time | Result |" >> "$RESULTS"
echo "|------|------|--------|" >> "$RESULTS"

cat > "$TMPDIR/list.ky" << 'KYEOF'
fn main() i32:
    lst = []
    i: &i32 = 0
    while i < 100000:
        push(lst, i)
        i = i + 1
    print(list_len(lst))
    print("\n")
    0
KYEOF
cat > "$TMPDIR/list.rs" << 'EOF'
fn main() { let mut v = Vec::new(); for i in 0..100000 { v.push(i); } println!("{}", v.len()); }
EOF

KY_L=$(build_ky "$TMPDIR/list.ky")
RS_L=$(build_rs "$TMPDIR/list.rs")
echo "| Kyle | $(bench "$KY_L") |" >> "$RESULTS"
# Verify binary exists
[ -x "$KY_L" ] || echo "DEBUG: list binary not found at $KY_L" >&2
echo "| Rust | $(bench "$RS_L") |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "## 5. Mandelbrot (float)" >> "$RESULTS"
echo "| Lang | Time |" >> "$RESULTS"
echo "|------|------|" >> "$RESULTS"

cp "$KY_SRC/examples/bench/mandelbrot.ky" "$TMPDIR/."
cp "$KY_SRC/examples/bench/mandelbrot.rs" "$TMPDIR/."
KY_M=$(build_ky "$TMPDIR/mandelbrot.ky")
RS_M=$(build_rs "$TMPDIR/mandelbrot.rs")
echo "| Kyle | $(bench "$KY_M") |" >> "$RESULTS"
echo "| Rust | $(bench "$RS_M") |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "## 6. Compilation Memory (primes)" >> "$RESULTS"
echo "| Lang | Peak Memory |" >> "$RESULTS"
echo "|------|-------------|" >> "$RESULTS"
KY_MEM=$(/usr/bin/time -l "$KY" build "$TMPDIR/primes.ky" 2>&1 | grep "peak memory" | awk '{print $1}')
RS_MEM=$(/usr/bin/time -l rustc -O "$TMPDIR/primes.rs" -o /dev/null 2>&1 | grep "peak memory" | awk '{print $1}')
echo "| Kyle | ${KY_MEM:-N/A} |" >> "$RESULTS"
echo "| Rust | ${RS_MEM:-N/A} |" >> "$RESULTS"

# ═══════════════════════════════════════════════════
echo "" >> "$RESULTS"
echo "---" >> "$RESULTS"
echo "## Features NOT benchmarked" >> "$RESULTS"
echo "| Feature | Why | ETA |" >> "$RESULTS"
echo "|---------|-----|-----|" >> "$RESULTS"
echo "| SIMD (AVX, NEON) | Kyle no expone intrinsics | 📅 Fase 19 |" >> "$RESULTS"
echo "| Concurrency (threads, mutex) | No implementado en Kyle | 📅 Fase D |" >> "$RESULTS"
echo "| Async/Await | Runtime existe, no expuesto | 📅 Fase D |" >> "$RESULTS"
echo "| TCP/UDP/HTTP Server | Packages en desarrollo | 📅 Fase 4-5 |" >> "$RESULTS"
echo "| HashMap avanzado | Dict → i64 limitado | 📅 Fase C |" >> "$RESULTS"
echo "| Regex/Crypto/Compression | No existen packages | 📅 Futuro |" >> "$RESULTS"
echo "| PGO | No implementado en toolchain | 📅 Futuro |" >> "$RESULTS"
echo "| Cache Miss, IPC, Branch Miss | Requiere perf (Linux) | 📅 Futuro |" >> "$RESULTS"
echo "| WebSocket/SSE | No implementado | 📅 Fase 5 |" >> "$RESULTS"
echo "| LLVM Vectorization control | Kyle no expone atributos | 📅 Futuro |" >> "$RESULTS"
echo "| Arena/Pool allocators | No implementados | 📅 Futuro |" >> "$RESULTS"
echo "" >> "$RESULTS"

rm -rf "$TMPDIR"
echo "Done. Results: $RESULTS"
