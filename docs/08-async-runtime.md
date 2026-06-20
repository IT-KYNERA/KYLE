# Kyle Async Runtime Specification v1.0

---

# Purpose

The Async Runtime is responsible for executing asynchronous
operations without blocking the current thread.

Examples:

- Network requests
- Database operations
- File operations
- Timers
- Background tasks
- Concurrent workloads

---

# Design Goals

```text
Simple

Predictable

Fast

No hidden threads

No callbacks

No promises

No futures exposed to the user

Python-like syntax

Rust-level performance
```

---

# Core Keywords

```kl
async

await
```

Only two keywords exist.

---

# Async Function

Declaration:

```kl
async fn load_users() -> [User]:

    ...
```

Call:

```kl
users = await load_users()
```

---

# Async Execution Model

When an async function is called:

```kl
load_users()
```

Nothing executes immediately.

The compiler creates an Async Task.

Conceptually:

```text
AsyncTask<User>
```

Internally.

Users never see this type.

---

# Await

```kl
users = await load_users()
```

Meaning:

```text
Suspend current execution

Wait until task completes

Resume execution

Return value
```

---

# Async Task

Task creation:

```kl
task = async load_users()
```

Meaning:

```text
Start execution in background

Return task handle
```

---

# Waiting For Task

```kl
users = await task
```

The runtime waits for completion.

---

# Multiple Tasks

```kl
users_task = async load_users()

posts_task = async load_posts()

users = await users_task

posts = await posts_task
```

Tasks execute concurrently.

---

# Fire And Forget

```kl
async send_metrics()
```

Meaning:

```text
Run in background

Ignore result
```

---

# Task Type

Internal compiler type:

```text
Task<T>
```

Examples:

```text
Task<User>

Task<[User]>

Task<String>
```

Not directly exposed.

---

# Async Return Types

```kl
async fn load() -> User
```

Compiler transforms internally:

```text
Task<User>
```

User still sees:

```kl
User
```

---

# Async Errors

Async functions may fail.

Example:

```kl
async fn load_user(id: i32) -> User!
```

Usage:

```kl
result = await load_user(1)

match result:

    ok(user):

        print(user.name)

    error(err):

        print(err)
```

---

# Scheduler

Kyle runtime includes a scheduler.

Responsibilities:

```text
Task creation

Task execution

Task suspension

Task wake-up

Task completion
```

---

# Scheduling Strategy

Version 1:

```text
Work-stealing scheduler
```

Inspired by:

```text
Tokio

Rayon

Go Runtime
```

Benefits:

```text
Scalable

Multi-core

Low overhead
```

---

# Runtime Threads

Default:

```text
One worker thread per CPU core
```

Example:

```text
8-core CPU

=> 8 worker threads
```

---

# Yielding

Long-running async code may yield.

Syntax:

```kl
await yield()
```

Meaning:

```text
Pause current task

Allow scheduler to execute others
```

---

# Sleeping

```kl
await sleep(1000)
```

Unit:

```text
Milliseconds
```

Example:

```kl
await sleep(500)
```

---

# Timeout

```kl
await timeout(
    load_users(),
    5000
)
```

Meaning:

```text
Fail if operation exceeds 5 seconds
```

---

# Cancellation

Create task:

```kl
task = async load_users()
```

Cancel:

```kl
task.cancel()
```

Check:

```kl
if task.cancelled:

    return
```

---

# Task Status

Possible states:

```text
Pending

Running

Completed

Failed

Cancelled
```

---

# Channels

Channels provide communication between tasks.

Create:

```kl
channel = Channel<i32>()
```

Send:

```kl
await channel.send(10)
```

Receive:

```kl
value = await channel.receive()
```

---

# Buffered Channels

```kl
channel = Channel<i32>(100)
```

Meaning:

```text
Buffer size = 100 messages
```

---

# Select

Wait for first available event.

```kl
match select:

    channel_a.receive():

        print("A")

    channel_b.receive():

        print("B")
```

Similar to:

```text
Go select

Rust tokio::select
```

---

# Async Streams

Return multiple values over time.

Declaration:

```kl
async fn events() -> stream<Event>
```

Usage:

```kl
for event in await events():

    print(event)
```

---

# Async Iterator

Example:

```kl
async fn users() -> stream<User>
```

Consume:

```kl
for user in await users():

    print(user.name)
```

---

# Runtime Memory Safety

Rules:

```text
No shared mutable state by default

Immutable by default

Message passing preferred

Channels over locks
```

---

# Mutex

Available when needed.

```kl
users = Mutex<[User]>()
```

Usage:

```kl
lock = await users.lock()

lock.add(user)
```

---

# Atomic Types

Supported:

```kl
AtomicBool

AtomicI32

AtomicI64

AtomicU32

AtomicU64
```

---

# Runtime Modules

```text
std.async

std.task

std.time

std.channel

std.sync
```

---

# Future Optimizations

Phase 1

```text
Single machine concurrency
```

Phase 2

```text
High-performance networking
```

Phase 3

```text
Distributed tasks
```

Phase 4

```text
Actor system
```

Phase 5

```text
Cluster scheduler
```

---

# Core Principle

```text
Users write:

async
await

The runtime handles everything else.
```

---

# Example

```kl
async fn load_user(id: i32) -> User!:

    ...

async fn load_orders(id: i32) -> [Order]!:

    ...

user_task = async load_user(1)

orders_task = async load_orders(1)

user = await user_task

orders = await orders_task

print(user.name)

print(orders.length)
```

---

# Runtime Philosophy

```text
Simple like C#

Readable like Python

Explicit like Rust

Fast like Go
```
