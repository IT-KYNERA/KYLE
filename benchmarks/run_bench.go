package main

import (
	"fmt"
	"os/exec"
	"strings"
	"time"
)

func bench(name string, cmd *exec.Cmd, warmup, runs int) time.Duration {
	for i := 0; i < warmup; i++ {
		c := exec.Command(cmd.Args[0], cmd.Args[1:]...)
		c.Stdout = nil
		c.Stderr = nil
		c.Run()
	}
	var total time.Duration
	for i := 0; i < runs; i++ {
		c := exec.Command(cmd.Args[0], cmd.Args[1:]...)
		c.Stdout = nil
		c.Stderr = nil
		start := time.Now()
		c.Run()
		total += time.Since(start)
	}
	return total / time.Duration(runs)
}

func main() {
	benches := []struct {
		name string
		cmd  *exec.Cmd
	}{
		{"C fib",       exec.Command("./fib/fib_c")},
		{"Rust fib",    exec.Command("./fib/fib_rs")},
		{"Go fib",      exec.Command("./fib/fib_go")},
		{"C# fib",      exec.Command("dotnet", "exec", "fib/fib.dll")},
		{"Java fib",    exec.Command("java", "-cp", "fib", "fib")},
		{"Kyle fib",    exec.Command("fib/fib_ky")},

		{"C concat",       exec.Command("./concat/concat_c")},
		{"Rust concat",    exec.Command("./concat/concat_rs")},
		{"Go concat",      exec.Command("./concat/concat_go")},
		{"C# concat",      exec.Command("dotnet", "exec", "concat/concat.dll")},
		{"Java concat",    exec.Command("java", "-cp", "concat", "concat")},
		{"Python concat",  exec.Command("python3", "concat/concat.py")},
		{"Kyle concat",    exec.Command("concat/concat_ky")},

		{"C primes",       exec.Command("./primes/primes_c")},
		{"Rust primes",    exec.Command("./primes/primes_rs")},
		{"Go primes",      exec.Command("./primes/primes_go")},
		{"C# primes",      exec.Command("dotnet", "exec", "primes/primes.dll")},
		{"Java primes",    exec.Command("java", "-cp", "primes", "primes")},
		{"Python primes",  exec.Command("python3", "primes/primes.py")},
		{"Kyle primes",    exec.Command("primes/primes_ky")},

		{"C matmul",       exec.Command("./matmul/matmul_c")},
		{"Rust matmul",    exec.Command("./matmul/matmul_rs")},
		{"Go matmul",      exec.Command("./matmul/matmul_go")},
		{"C# matmul",      exec.Command("dotnet", "exec", "matmul/matmul.dll")},
		{"Java matmul",    exec.Command("java", "-cp", "matmul", "matmul")},
		{"Python matmul",  exec.Command("python3", "matmul/matmul.py")},
		{"Kyle matmul",    exec.Command("matmul/matmul_ky")},
	}

	warmup := 3
	runs := 5

	fmt.Println()
	fmt.Println("╔══════════════════════════════════════════════╗")
	fmt.Println("║     Kyle v0.5.3 — BENCHMARKS (Apple M3)    ║")
	fmt.Println("╠══════════════════════════════════════════════╣")
	fmt.Println()
	fmt.Printf("  Warmup: %d, Runs: %d\n", warmup, runs)
	fmt.Println()

	var prevGroup string
	for i, b := range benches {
		group := strings.Split(b.name, " ")[0]
		if i > 0 && group != prevGroup {
			fmt.Println()
		}
		prevGroup = group
		d := bench(b.name, b.cmd, warmup, runs)
		if d.Milliseconds() > 0 {
			fmt.Printf("  %-20s %7s\n", b.name+":", fmt.Sprintf("%dms", d.Milliseconds()))
		} else {
			fmt.Printf("  %-20s %7s\n", b.name+":", fmt.Sprintf("%dµs", d.Microseconds()))
		}
	}
	fmt.Println()
	fmt.Println("  Compiler: LLVM 18.1 / Apple Clang 17 (Kyle, C)")
	fmt.Println("  Runtime:  macOS 26.5 (Apple Silicon M3)")
	fmt.Println("└──────────────────────────────────────────────")
}
