# datetime — Fechas, Horas y Duraciones

> Módulo de tipos temporales: `date_time`, `date`, `time`, `duration`.
> Todos son nativos de Kyle (no requieren `from ... import`).

## date_time: fecha y hora completas

```ky
# DateTime (fecha + hora)
dt: date_time = date_time.now()
dt = date_time.parse("2024-01-01T12:30:00")
dt = date_time.from_ymdhms(2024, 1, 1, 12, 30, 0)

year: i32 = dt.year()
month: i32 = dt.month()
day: i32 = dt.day()
hour: i32 = dt.hour()
minute: i32 = dt.minute()
second: i32 = dt.second()

dt2: date_time = dt.add_days(7)
dt3: date_time = dt.add_hours(3)
diff: duration = dt.diff(dt2)
```

### Métodos de date_time

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `date_time.now()` | `date_time` | Fecha/hora actual |
| `date_time.parse(s)` | `date_time` | Parsear string ISO |
| `date_time.from_ymdhms(y, M, d, h, m, s)` | `date_time` | Construir desde componentes |
| `.year()` | `i32` | Año |
| `.month()` | `i32` | Mes (1-12) |
| `.day()` | `i32` | Día (1-31) |
| `.hour()` | `i32` | Hora (0-23) |
| `.minute()` | `i32` | Minuto (0-59) |
| `.second()` | `i32` | Segundo (0-59) |
| `.add_days(n)` | `date_time` | Sumar días |
| `.add_hours(n)` | `date_time` | Sumar horas |
| `.diff(other)` | `duration` | Diferencia entre fechas |
| `.format(fmt)` | `str` | Formatear con patrón |
| `.to_str()` | `str` | String ISO |

## date: solo fecha (sin hora)

```ky
d: date = date.today()
d = date.from_ymd(2024, 1, 1)
d = date.parse("2024-01-01")

year: i32 = d.year()
month: i32 = d.month()
day: i32 = d.day()
weekday: i32 = d.weekday()   # 0=domingo, 1=lunes...

d2: date = d.add_days(7)
```

### Métodos de date

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `date.today()` | `date` | Fecha actual |
| `date.from_ymd(y, M, d)` | `date` | Construir desde componentes |
| `date.parse(s)` | `date` | Parsear string |
| `.year()` | `i32` | Año |
| `.month()` | `i32` | Mes |
| `.day()` | `i32` | Día |
| `.weekday()` | `i32` | Día de semana |
| `.add_days(n)` | `date` | Sumar días |
| `.format(fmt)` | `str` | Formatear |
| `.to_str()` | `str` | String ISO |

## time: solo hora (sin fecha)

```ky
t: time = time.now()
t = time.from_hms(12, 30, 0)
t = time.parse("12:30:00")

hour: i32 = t.hour()
minute: i32 = t.minute()
second: i32 = t.second()
```

### Métodos de time

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `time.now()` | `time` | Hora actual |
| `time.from_hms(h, m, s)` | `time` | Construir desde componentes |
| `time.parse(s)` | `time` | Parsear string |
| `.hour()` | `i32` | Hora |
| `.minute()` | `i32` | Minuto |
| `.second()` | `i32` | Segundo |
| `.to_str()` | `str` | String HH:MM:SS |

## duration: intervalos de tiempo

```ky
d: duration = duration.from_secs(60)
d = duration.from_millis(1000)
d = duration.from_hours(1)
d = duration.from_days(7)

total_secs: i64 = d.total_seconds()
total_ms: i64 = d.total_milliseconds()
s: str = d.to_str()     # → "1h 0m 0s"
```

### Métodos de duration

| Método | Retorno | Descripción |
|--------|---------|-------------|
| `duration.from_secs(n)` | `duration` | Desde segundos |
| `duration.from_millis(n)` | `duration` | Desde milisegundos |
| `duration.from_hours(n)` | `duration` | Desde horas |
| `duration.from_days(n)` | `duration` | Desde días |
| `.total_seconds()` | `i64` | Total en segundos |
| `.total_milliseconds()` | `i64` | Total en ms |
| `.to_str()` | `str` | String legible |

### sleep (función global)

```ky
sleep(1000)    # pausa en milisegundos (función global)
```

### Ejemplo completo

```ky
# Medición de tiempo
start: date_time = date_time.now()
# ... código a medir ...
end: date_time = date_time.now()
elapsed: duration = start.diff(end)
println("tomó " + elapsed.total_milliseconds().to_str() + "ms")

# Fechas
hoy: date = date.today()
cumple: date = date.from_ymd(2024, 12, 25)
dias: i64 = hoy.diff(cumple).total_days()
println("faltan " + dias.to_str() + " días")

# Timer
sleep(500)   # 500ms
```
