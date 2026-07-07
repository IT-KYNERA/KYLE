# std.io — Input/Output

| Function | Description |
|----------|-------------|
| `print(value)` | Print without newline |
| `println(value)` | Print with newline |
| `input()` | Read line from stdin |
| `input(prompt)` | Show prompt, read line |
| `readFile(path)` | Read file as string |
| `write_file(path, data)` | Write string to file |

## File I/O

```ky
from std.io import readFile, write_file

data = readFile("config.txt")
write_file("output.txt", "hello")
```
