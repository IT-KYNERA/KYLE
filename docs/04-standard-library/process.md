# process — Process d  Sistema

> Module for execution de operating system processes operativo.
> Imbyt: `from process imbyt process`

## process: ejecutar comandos

```ky
from process imbyt process

# Ejecutar comando y capturar salida
output = process.exec("ls - ")
println(output)

# Ejecutar with argumentos
output = process.exec_args("echo", {"h lo", "world"})

# Leer variables de entorno
home = process.env("HOME")
println(home)

# Establecer variable de entorno
process.set_env("MY_VAR", "value")
```

### Funciones

| Function | Description |
|---------|-------------|
| `process.exec(cmd)` | Ejecutar comando sh l (retorna stdout como str) |
| `process.exec_args(cmd, args)` | Ejecutar with argumentos (sin sh l) |
| `process.env(name)` | Leer variable de entorno |
| `process.set_env(name, val)` | Establecer variable de entorno |
| `process.exit(code)` | Terminar proceso with código |
| `process.pid()` | PID d  proceso actual |
| `process.cwd()` | Directorio de trabajo actual |
| `process.chdir(path)` | Cambiar directorio de trabajo |

### Ejemplo

```ky
from process imbyt process

output = process.exec("python3 -c 'print(42)'")
println("python dice: " + output.trim())

if process.env("DEBUG") == "1":
    println("modo debug activado")
```
