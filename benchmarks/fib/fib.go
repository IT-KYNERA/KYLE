package main

func fib(n int32) int32 {
	var a, b int32 = 0, 1
	for i := int32(0); i < n; i++ {
		tmp := a + b
		a = b
		b = tmp
	}
	return b
}

func main() {
	println("Result:", fib(500000000))
}
