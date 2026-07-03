# Modules and Packages

## Modules

A module is a `.ky` file. The module name is the file name without extension.

```ky
# src/math.ky
fn square(x: i32) i32:
    x * x
```

## Imports

```ky
from math import square          # import single name
from math import *               # import all public names
from math import add, subtract   # import multiple
```

## Visibility

| Prefix | Scope |
|--------|-------|
| `name` | Public |
| `_name` | Protected (same package / subclasses) |
| `__name` | Private (same module) |

```ky
pub = 1          # visible everywhere
_prot = 2        # visible in package
__priv = 3       # visible in this module only
```

## Packages

A package is a directory with a `ky.toml` manifest.

```
my-project/
├── ky.toml
└── src/
    ├── main.ky
    ├── lib.ky
    └── helper.ky
```

## Package Manifest (`ky.toml`)

```toml
[project]
name = "my-project"
version = "0.1.0"
edition = "2024"

[dependencies]
ky-http = "1.0"
ky-json = "0.5"
```

## Standard Library

```ky
from std import io          # read_file, write_file
from std import json        # parse, stringify
from std import math        # pow, sqrt, gcd, clamp
from std import str         # starts_with, ends_with, capitalize
from std import time        # timestamp, sleep, seconds_since
from std import collections # list_sum, list_product
from std import testing     # assert, assert_eq
```
