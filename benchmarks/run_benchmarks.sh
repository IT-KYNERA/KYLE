#!/bin/bash
# =============================================================================
#  Kyle v0.6 — Multi-Language Benchmark Runner
#  Usage: bash run_benchmarks.sh
# =============================================================================
cd "$(dirname "$0")"
ROOT="$(cd .. && pwd)"

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; CYAN='\033[0;36m'; NC='\033[0m'

BENCHES=("primes" "fib" "concat" "matmul")
BNAMES=("Prime Sieve (3M)" "Fibonacci (500M)" "String Concat (500k)" "MatMul (100x100x10)")

echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              KYLE v0.6 — Multi-Language Benchmarks         ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo "  $(uname -srm) | $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "")"
echo ""

# ---------------------------------------------------------------------------
#  Measure helper
# ---------------------------------------------------------------------------
measure() {
    local cmd=$1 warm=${2:-3} runs=${3:-5} i total=0 count=0 to=0 ts elapsed
    for i in $(seq 1 $warm); do eval "$cmd" > /dev/null 2>&1; done
    for i in $(seq 1 $runs); do
        ts=$(( $(date +%s%N) ))
        eval "$cmd" > /dev/null 2>&1
        elapsed=$(( ($(date +%s%N) - ts) / 1000000 ))
        [ $elapsed -gt 120000 ] && { to=1; break; }
        total=$((total + elapsed)); count=$((count + 1))
    done
    [ $to -eq 1 ] && { echo "TO"; return; }
    [ $count -gt 0 ] && echo $((total / count)) || echo "-"
}

# ---------------------------------------------------------------------------
#  Build everything
# ---------------------------------------------------------------------------
echo " Building..."
# C
for b in "${BENCHES[@]}"; do gcc -O3 -o "$b/${b}_c" "$b/$b.c" -lm 2>/dev/null; done
# C++
for b in "${BENCHES[@]}"; do g++ -O3 -o "$b/${b}_cpp" "$b/$b.cpp" -lm 2>/dev/null; done
# Rust
for b in "${BENCHES[@]}"; do rustc -C opt-level=3 -o "$b/${b}_rs" "$b/$b.rs" 2>/dev/null; done
# Go
command -v go &>/dev/null && for b in "${BENCHES[@]}"; do go build -o "$b/${b}_go" "$b/$b.go" 2>/dev/null; done
# C#
command -v dotnet &>/dev/null && for b in "${BENCHES[@]}"; do
    (cd "$b" && dotnet build -c Release -nologo -v q 2>/dev/null)
    cp "$b/bin/Release/net10.0/$b" "$b/${b}_cs" 2>/dev/null
done 2>/dev/null
# Java
command -v javac &>/dev/null && for b in "${BENCHES[@]}"; do javac "$b/$b.java" -d "$b" 2>/dev/null; done
# Kyle
ky="$ROOT/target/release/ky"
[ ! -f "$ky" ] && (cd "$ROOT" && cargo build --release --bin ky 2>/dev/null)
if [ -f "$ky" ]; then
    for b in "${BENCHES[@]}"; do
        $ky build "$b/$b.ky" 2>/dev/null
        cp "$ROOT/target/debug/$b" "$b/${b}_ky" 2>/dev/null
    done
fi

# ---------------------------------------------------------------------------
#  Auto-detect available languages
# ---------------------------------------------------------------------------
LANGS=()
try_lang() {
    local lang=$1 bin=$2
    [ -f "$bin" ] || return 1
    LANGS+=("$lang")
}
try_lang "C"      "primes/primes_c"
try_lang "C++"    "primes/primes_cpp"
try_lang "Rust"   "primes/primes_rs"
try_lang "C#"     "primes/primes_cs"
command -v go     &>/dev/null && try_lang "Go"     "primes/primes_go"
command -v java   &>/dev/null && try_lang "Java"   "primes/primes.class"
command -v python3 &>/dev/null && try_lang "Python" "primes/primes.py"
try_lang "Kyle"   "primes/primes_ky"

# ---------------------------------------------------------------------------
#  Run all
# ---------------------------------------------------------------------------
echo ""
echo " Running..."
echo ""

# Table header
printf "│ %-24s" "Benchmark"
for l in "${LANGS[@]}"; do printf " │ %7s" "$l"; done
echo " │"
printf "│ %-24s" "$(printf '─%.0s' $(seq 1 24))"
for l in "${LANGS[@]}"; do printf " │ %7s" "───────"; done
echo " │"

for bi in "${!BENCHES[@]}"; do
    b=${BENCHES[$bi]}; bn=${BNAMES[$bi]}
    printf "│ %-24s" "$bn"
    
    for l in "${LANGS[@]}"; do
        case "$l" in
            C)      r=$(measure "$b/${b}_c") ;;
            C++)    r=$(measure "$b/${b}_cpp") ;;
            Rust)   r=$(measure "$b/${b}_rs") ;;
            C#)     r=$(measure "dotnet exec $b/bin/Release/net10.0/$b.dll") ;;
            Go)     r=$(measure "$b/${b}_go") ;;
            Java)   r=$(measure "java -cp $b $b" 2 3) ;;
            Python) [ "$b" = "fib" ] && r="TO" || r=$(measure "python3 $b/$b.py" 2 3) ;;
            Kyle)   r=$(measure "$b/${b}_ky") ;;
            *)      r="-" ;;
        esac
        printf " │ %7s" "${r}ms"
    done
    echo " │"
done

echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo " Done.  (C/C++/Rust -O3, Go native, C# Release, Java, Python3, Kyle debug)"
echo ""
