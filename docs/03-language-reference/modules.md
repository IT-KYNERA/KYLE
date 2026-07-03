# Modules

## Module definition

A module is a `.ky` file. The module name is the filename without extension.

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

## Packages

A package is a directory with a `ky.toml` manifest.

```
my-project/
├── ky.toml
└── src/
    ├── main.ky
    └── lib.ky
```

## Package manifest (ky.toml)

```toml
[project]
name = "my-project"
version = "0.1.0"
edition = "2024"

[dependencies]
ky-http = "1.0"
```

## Standard library

```ky
from std import io
from std import json
from std import math
from std import time
```
