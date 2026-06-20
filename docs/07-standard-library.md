# Kyle Standard Library Specification v1.0

---

# Philosophy

The Kyle Standard Library is designed around:

```text
Minimal

Fast

Safe

Predictable

Cross Platform

Strongly Typed

Easy To Learn

Enterprise Ready
```

---

# Design Goals

```text
Small Core

No Hidden Allocations

Consistent APIs

Explicit Errors

Async First

Modern Naming

High Performance
```

---

# Root Modules

Kyle v2.0 ships with:

```text
io        ✅ implemented  (std/io.kl)
math      ✅ implemented  (std/math.kl)
testing   ✅ implemented  (std/testing.kl)
core      ✅ implemented  (std/core.kl)
json      🔶 planned
net       🔶 planned
time      🔶 planned
filesystem 🔶 planned
collections 🔶 planned
crypto    🔶 planned
async     🔶 planned
```

---

# Module Structure

```text
std/

├── core.kl      ✅ utility functions
├── io.kl        ✅ I/O wrappers (print, println, read_line)
├── math.kl      ✅ abs, pow, sqrt, gcd
├── testing.kl   ✅ assert, assert_eq, assert_str
├── json.kl      🔶 planned
├── net          🔶 planned
├── time         🔶 planned
├── filesystem   🔶 planned
├── collections  🔶 planned
├── crypto       🔶 planned
└── async        🔶 planned
```

---

# IO Module

Import:

```kl
import io
```

Purpose:

```text
Console Input

Console Output

Streams
```

---

## Print

```kl
io.print("Hello")
```

---

## Print Line

```kl
io.println("Hello")
```

Output:

```text
Hello
```

---

## Read Line

```kl
name = io.read_line()
```

---

## Read Integer

```kl
age = io.read_i32()
```

---

## Read Float

```kl
salary = io.read_f64()
```

## Top-Level Built-in I/O

These functions are available without any import:

```kl
print(value)              # print without newline
println(value)            # print with newline
input(prompt)             # → str: read line from stdin
open(path, mode)          # → i64: file handle (mode: 0=read, 1=write)
read_str(handle)          # → str: read entire file
write_str(handle, text)   # write string to file
close(handle)             # close file handle
```

---

# Math Module

Import:

```kl
import math
```

---

## Constants

```kl
math.pi

math.e
```

---

## Basic Functions

```kl
math.abs(x)

math.min(a, b)

math.max(a, b)

math.clamp(value, min, max)
```

---

## Rounding

```kl
math.floor(x)

math.ceil(x)

math.round(x)
```

---

## Powers

```kl
math.pow(x, y)

math.sqrt(x)
```

---

## Trigonometry

```kl
math.sin(x)

math.cos(x)

math.tan(x)
```

---

## Random

```kl
math.random()
```

Returns:

```text
0.0 -> 1.0
```

---

# JSON Module

Import:

```kl
import json
```

Purpose:

```text
Serialization

Deserialization
```

---

## Parse

```kl
user = json.parse(text)?
```

---

## Stringify

```kl
text = json.stringify(user)
```

---

## Pretty Format

```kl
text = json.pretty(user)
```

---

# Network Module

Import:

```kl
import net
```

---

# HTTP Client

GET:

```kl
response = await net.get(
    "https://api.company.com"
)?
```

---

POST:

```kl
response = await net.post(
    url,
    body
)?
```

---

PUT:

```kl
response = await net.put(
    url,
    body
)?
```

---

DELETE:

```kl
response = await net.delete(
    url
)?
```

---

# HTTP Response

```kl
response.status

response.body

response.headers
```

---

# Time Module

🔶 Planned for `import time`. Currently `now()` and `sleep()` are top-level built-ins:

## Top-Level Time Functions

Available without import:

```kl
now()                     # → i64: Unix timestamp in milliseconds
sleep(ms)                 # sleep for milliseconds (blocking)
```

---

# Filesystem Module

Import:

```kl
import filesystem
```

---

# Read File

```kl
content = filesystem.read(
    "data.txt"
)?
```

---

# Write File

```kl
filesystem.write(
    "data.txt",
    content
)?
```

---

# Append File

```kl
filesystem.append(
    "log.txt",
    message
)?
```

---

# Delete File

```kl
filesystem.delete(
    "temp.txt"
)?
```

---

# Check Existence

```kl
exists = filesystem.exists(
    "data.txt"
)
```

---

# Create Directory

```kl
filesystem.mkdir(
    "uploads"
)?
```

---

# Collections Module

Import:

```kl
import collections
```

Purpose:

```text
Advanced Data Structures
```

---

# List

```kl
users: list<User>
```

Methods:

```kl
users.add(user)

users.remove(user)

users.clear()

users.count()
```

---

# Dictionary

```kl
dict<str, User>
```

Methods:

```kl
users.get("admin")

users.contains("admin")

users.remove("admin")
```

---

# Set

```kl
set<str>
```

Methods:

```kl
roles.add("admin")

roles.contains("admin")
```

---

# Queue

```kl
queue<User>
```

Methods:

```kl
enqueue()

dequeue()

peek()
```

---

# Stack

```kl
stack<User>
```

Methods:

```kl
push()

pop()

peek()
```

---

# Iterator Protocol

Any type that implements the `Iterable` contract can be used in a `for` loop:

```kl
contract Iterable<T>:

    fn iterator() -> Iterator<T>

contract Iterator<T>:

    fn next() -> Option<T>
```

Built-in types implementing `Iterable`:

```kl
list<T>
str
Dict<K, V>
Set<T>
Range
```

Custom iteration:

```kl
for item in my_collection:

    process(item)
```

Equivalent to:

```kl
it = my_collection.iterator()

while item = it.next():

    process(item)
```

---

# Crypto Module

Import:

```kl
import crypto
```

---

# Hashing

```kl
hash = crypto.sha256(text)

hash = crypto.sha512(text)
```

---

# Random Bytes

```kl
token = crypto.random(32)
```

---

# UUID

```kl
id = crypto.uuid()
```

---

# Password Hashing

```kl
hash = crypto.password(password)
```

Verify:

```kl
valid = crypto.verify(
    password,
    hash
)
```

---

# Async Module

Import:

```kl
import async
```

Purpose:

```text
Concurrency

Task Management
```

---

# Create Task

```kl
task = async.run(

    load_users()
)
```

---

# Await Task

```kl
users = await task
```

---

# Parallel Execution

```kl
users_task = async.run(
    load_users()
)

roles_task = async.run(
    load_roles()
)

users = await users_task

roles = await roles_task
```

---

# Wait All

```kl
results = await async.all(

    users_task,

    roles_task
)
```

---

# Testing Module

Import:

```kl
import testing
```

## Assertions

```kl
testing.assert(condition)              # assert truthy
testing.assert_eq(a, b)                # assert i32 equality
testing.assert_str(a, b)               # assert string equality
```

---

# Core Runtime Types

Available everywhere:

```kl
str

bool

char

i8
i16
i32
i64

u8
u16
u32
u64

f32
f64
```

---

# Built-In Collections

Available without imports:

```kl
list<T>

dict<K, V>

set<T>

tuple
```

---

# Built-In Functions

Available without imports:

## String & Character

```kl
len(s)                    # → i32: length of string s
str(value)                # → str: convert i32/f64/bool to string
char_at(s, i)             # → char (i8): character at index i
ord(c)                    # → i32: Unicode codepoint of char
is_digit(c)               # → i32: 1 if '0'-'9'
is_alpha(c)               # → i32: 1 if 'a'-'z' or 'A'-'Z'
is_alnum(c)               # → i32: 1 if alphanumeric
is_whitespace(c)          # → i32: 1 if space/tab/newline
is_upper(c)               # → i32: 1 if uppercase letter
is_lower(c)               # → i32: 1 if lowercase letter
contains(s, sub)          # → i32: 1 if s contains sub
to_upper(s)               # → str: uppercase copy
to_lower(s)               # → str: lowercase copy
trim(s)                   # → str: trim whitespace
replace(s, from, to)      # → str: replace all occurrences
```

## I/O

```kl
print(value)              # print without newline
println(value)            # print with newline
input(prompt)             # → str: read line from stdin
open(path, mode)          # → i64: file handle (mode: 0=read, 1=write)
read_str(handle)          # → str: read entire file
write_str(handle, text)   # write string to file
close(handle)             # close file handle
```

## Time

```kl
sleep(ms)                 # sleep for milliseconds
now()                     # → i64: Unix timestamp in milliseconds
```

## Math

```kl
range(end)                # returns a range value
range(start, end)         # returns a range value
```

---

# Error Types

Provided by standard library:

```text
Error

IoError

FileError

NetworkError

TimeoutError

JsonError

ParseError

PermissionError
```

---

# Future Modules

Planned:

```text
database

graphql

grpc

websocket

compression

image

audio

video

machine_learning
```

---

# Enterprise Modules

Future Enterprise SDK:

```text
cloud

identity

messaging

storage

monitoring

telemetry
```

---

# Design Principles

```text
Small Core

Fast Startup

Explicit Errors

Strong Typing

Async Ready

Cross Platform

Enterprise Friendly
```

---

# Version

```text
Kyle Standard Library Specification v2.0
Last updated: 2026-11-19
```
