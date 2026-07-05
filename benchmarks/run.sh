#!/bin/bash
# Kyle Language Benchmark Runner
# Usage: bash run.sh

cd "$(dirname "$0")"

run_bench() {
    local label=$1
    local cmd=$2
    local warmup=$3
    local runs=$4
    python3 -c "
import subprocess, time
times = []
for _ in range($warmup + $runs):
    start = time.time()
    subprocess.run('$cmd', shell=True, capture_output=True)
    times.append(time.time() - start)
times = times[$warmup:]
avg = sum(times) / len(times)
print(f'{avg*1000:8.1f}ms')
" 2>/dev/null
}

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║         KYLE v0.5 BENCHMARKS (Apple M1/macOS)   ║"
echo "╠══════════════════════════════════════════════════╣"
echo ""
echo "┌─ Prime Sieve (1..1,000,000)"
printf "│  %-18s %s\n" "C (gcc -O3)"   "$(run_bench C ./primes/primes_c 3 5)"
printf "│  %-18s %s\n" "Rust (opt=3)"  "$(run_bench Rust ./primes/primes_rs 3 5)"
printf "│  %-18s %s\n" "C# (.NET 9)"   "$(run_bench CSharp ./primes/primes_cs 3 5)"
printf "│  %-18s %s\n" "Java 21"       "$(run_bench Java 'java -cp primes primes' 3 5)"
printf "│  %-18s %s\n" "Python 3.14"   "$(run_bench Python 'python3 primes/primes.py' 2 3)"
printf "│  %-18s %s\n" "Kyle 0.5"      "$(run_bench Kyle ./primes/primes_ky 3 5)"
echo ""
echo "┌─ Fibonacci Loop (10,000,000 iterations)"
printf "│  %-18s %s\n" "C (gcc -O3)"   "$(run_bench C ./fib/fib_c 3 5)"
printf "│  %-18s %s\n" "Rust (opt=3)"  "$(run_bench Rust ./fib/fib_rs 3 5)"
printf "│  %-18s %s\n" "Kyle 0.5"      "$(run_bench Kyle ./fib/fib_ky 3 5)"
echo ""
echo "┌─ String Concat (50,000 operations)"
printf "│  %-18s %s\n" "C (gcc -O3)"   "$(run_bench C ./concat/concat_c 2 5)"
printf "│  %-18s %s\n" "Rust (opt=3)"  "$(run_bench Rust ./concat/concat_rs 2 5)"
printf "│  %-18s %s\n" "Kyle 0.5"      "$(run_bench Kyle ./concat/concat_ky 2 5)"
echo ""
echo "╚══════════════════════════════════════════════════╝"
