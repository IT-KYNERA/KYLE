#!/bin/bash
# Benchmark runner for Kyle v0.5
# Usage: bash run.sh

cd "$(dirname "$0")"

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║       KYLE v0.5 — BENCHMARKS (Apple M1/macOS)  ║"
echo "╠══════════════════════════════════════════════════╣"
echo ""
echo "  Los benchmarks se ejecutan con:"
echo "  3 warmup + 5 mediciones, usando /usr/bin/time"
echo ""

bench() {
    local name=$1
    local cmd=$2
    local warmup=${3:-3}
    local runs=${4:-5}
    for i in $(seq 1 $warmup); do eval $cmd > /dev/null 2>&1; done
    total=0
    for i in $(seq 1 $runs); do
        t=$(/usr/bin/time -p bash -c "$cmd" 2>&1 | grep real | awk '{print $2}')
        ms=$(python3 -c "print(int(round(float('$t')*1000)))")
        total=$((total + ms))
    done
    echo $((total / runs))
}

printf "│  %-22s %s  %s  %s  %s  %s  %s\n" "" "C" "Rust" "C#" "Java" "Python" "Kyle"
echo "├──────────────────────────────────────────────────────────────"

t_c=$(bench "C" "./primes/primes_c")
t_rs=$(bench "Rust" "./primes/primes_rs")
t_cs=$(bench "C#" "./primes/primes_cs")
t_jv=$(bench "Java" "java -cp primes primes")
t_py=$(bench "Python" "python3 primes/primes.py" 2 3)
t_kl=$(bench "Kyle" "./primes/primes_ky")
printf "│  %-22s %4sms %5sms %5sms %5sms %6sms %5sms\n" "Prime Sieve (1M)" "$t_c" "$t_rs" "$t_cs" "$t_jv" "$t_py" "$t_kl"

echo "├──────────────────────────────────────────────────────────────"

t_c=$(bench "C" "./fib/fib_c")
t_rs=$(bench "Rust" "./fib/fib_rs")
t_cs=$(bench "C#" "dotnet exec ./fib/fib_cs.dll")
t_jv=$(bench "Java" "java -cp ./fib fib")
t_py=$(bench "Python" "python3 ./fib/fib.py" 2 3)
t_kl=$(bench "Kyle" "./fib/fib_ky")
printf "│  %-22s %4sms %5sms %5sms %5sms %6sms %5sms\n" "Fibonacci (10M)" "$t_c" "$t_rs" "$t_cs" "$t_jv" "$t_py" "$t_kl"

echo "├──────────────────────────────────────────────────────────────"

t_c=$(bench "C" "./concat/concat_c" 2 5)
t_rs=$(bench "Rust" "./concat/concat_rs" 2 5)
t_cs=$(bench "C#" "dotnet exec ./concat/concat_cs.dll" 2 5)
t_jv=$(bench "Java" "java -cp ./concat concat" 2 5)
t_py=$(bench "Python" "python3 ./concat/concat.py" 2 3)
t_kl=$(bench "Kyle" "./concat/concat_ky" 2 5)
printf "│  %-22s %4sms %5sms %5sms %5sms %6sms %5sms\n" "String Concat (50k)" "$t_c" "$t_rs" "$t_cs" "$t_jv" "$t_py" "$t_kl"

echo "└──────────────────────────────────────────────────────────────"
