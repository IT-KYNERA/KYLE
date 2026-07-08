# process — Procesos del Sistema

> Módulo de ejecución de procesos del sistema operativo.
> Import: `from process import process`

## process: ejecutar comandos

```ky
from process import process

# Ejecutar comando y capturar salida
output = process.exec("ls -la")
println(output)

# Ejecutar con argumentos
output = process.exec_args("echo", {"hello", "world"})

# Leer variables de entorno
home = process.env("HOME")
println(home)

# Establecer variable de entorno
process.set_env("MY_VAR", "value")
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `process.exec(cmd)` | Ejecutar comando shell (retorna stdout como str) |
| `process.exec_args(cmd, args)` | Ejecutar con argumentos (sin shell) |
| `process.env(name)` | Leer variable de entorno |
| `process.set_env(name, val)` | Establecer variable de entorno |
| `process.exit(code)` | Terminar proceso con código |
| `process.pid()` | PID del proceso actual |
| `process.cwd()` | Directorio de trabajo actual |
| `process.chdir(path)` | Cambiar directorio de trabajo |

### Ejemplo

```ky
from process import process

output = process.exec("python3 -c 'print(42)'")
println("python dice: " + output.trim())

if process.env("DEBUG") == "1":
    println("modo debug activado")
```
