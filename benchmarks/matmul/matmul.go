package main

func main() {
	n := 100
	a := make([]int64, n*n)
	b := make([]int64, n*n)
	c := make([]int64, n*n)
	for i := 0; i < n*n; i++ {
		a[i] = 1
		b[i] = 2
	}
	for iter := 0; iter < 30; iter++ {
		for i := 0; i < n; i++ {
			for j := 0; j < n; j++ {
				sum := int64(0)
				for k := 0; k < n; k++ {
					sum += a[i*n+k] * b[k*n+j]
				}
				c[i*n+j] = sum
			}
		}
	}
	println("Matmul done:", c[n*n-1])
}
