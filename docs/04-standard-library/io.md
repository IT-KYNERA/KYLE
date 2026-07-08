# std.io — Input/Output

| Function | Description |
|----------|-------------|
| `print(value)` | Print without newline |
| `println(value)` | Print with newline |
| `input()` | Read line from stdin |
| `input(prompt)` | Show prompt, read line |
| `read_file(path)` | Read file as string |
| `write_file(path, data)` | Write string to file |

## file I/O

```ky
from std.io import read_file, write_file

data = read_file("config.txt")
write_file("output.txt", "hello")
```
