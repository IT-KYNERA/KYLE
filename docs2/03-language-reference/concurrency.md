# Concurrency

## Async functions

```ky
async fn fetch_data() str:
    # ... async operation ...
    "data"
```

## Async expression

```ky
task = async fetch_data()
```

## Await

```ky
result = await task
```

## Threads

```ky
handle = spawn_thread(fn():
    # ... code in another thread ...
)

result = join_thread(handle)
```

## Channels

```ky
chan = channel()

send(chan, "hello")
msg = recv(chan)
```
