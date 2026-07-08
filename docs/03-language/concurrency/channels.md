# Channels

> Comunicación entre hilos mediante canales. Actualmente solo `i64`.
> Los canales tienen buffer y soportan `send`/`recv` bloqueantes.

## Uso básico

```ky
ch: i64 = channel_new(16)       # canal con buffer de 16
channel_send(ch, 42)
val: i64 = channel_recv(ch)
println(val.to_str())            # 42
channel_free(ch)
```

## Productor-Consumidor

```ky
fn productor(ch: i64):
    i: ^i64 = 0
    while i < 10:
        channel_send(ch, i)
        i = i + 1
    channel_close(ch)

fn consumidor(ch: i64):
    loop:
        val: i64 = channel_recv(ch)
        if val == -1:    # señal de cierre
            break
        println(val.to_str())

fn main() i32:
    ch: i64 = channel_new(16)
    h1: i64 = spawn_thread(productor as ptr, ch)
    h2: i64 = spawn_thread(consumidor as ptr, ch)
    join_thread(h1)
    join_thread(h2)
    channel_free(ch)
    0
```

## Funciones

| Función | Descripción |
|---------|-------------|
| `channel_new(capacity)` | Crear canal con buffer |
| `channel_send(ch, val)` | Enviar (bloquea si lleno) |
| `channel_recv(ch)` | Recibir (bloquea si vacío) |
| `channel_close(ch)` | Cerrar canal |
| `channel_len(ch)` | Elementos en buffer |
| `channel_free(ch)` | Liberar canal |

## Limitaciones actuales

- Solo `i64` como tipo de valor
- Sin `select` para multiplexar canales
- Canales sin tipado genérico (`channel<T>`)
- Futuro: `channel<T>`, `select`

## Ver también

- `threads.md` — Hilos del SO
- `synchronization.md` — Mutex, barriers
