package main

import "strings"

func main() {
	var sb strings.Builder
	for i := 0; i < 500000; i++ {
		sb.WriteByte('x')
	}
	println(sb.Len())
}
