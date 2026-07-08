# 04-standard-library

> Librería estándar de Kyle. Cada módulo es un namespace que se importa y contiene
> funciones, tipos y utilidades organizadas jerárquicamente.

## Módulos

| Módulo | Import | Descripción |
|--------|--------|-------------|
| `core` | `from core import ...` | Tipos fundamentales (`option`, `result`) |
| `collections` | `from collections import ...` | Listas, dicts, sets |
| `strings` | `from strings import ...` | Utilidades de string |
| `io` | `from io import ...` | Entrada/salida por consola |
| `fs` | `from fs import ...` | Sistema de archivos |
| `path` | `from path import ...` | Manipulación de rutas |
| `net` | `from net import ...` | Red (TCP, UDP) |
| `http` | `from http import ...` | HTTP client/server |
| `json` | `from json import ...` | JSON |
| `xml` | `from xml import ...` | XML |
| `math` | `from math import ...` | Matemáticas |
| `random` | `from random import ...` | Aleatoriedad |
| `time` | `from time import ...` | Tiempo y sleep |
| `datetime` | `from datetime import ...` | Fechas y duraciones |
| `process` | `from process import ...` | Procesos del SO |
| `thread` | `from thread import ...` | Hilos |
| `sync` | `from sync import ...` | Sincronización |
| `crypto` | `from crypto import ...` | Criptografía |
| `regex` | `from regex import ...` | Expresiones regulares |
| `serialization` | `from serialization import ...` | Serialización |
| `database` | `from database import ...` | Base de datos |
| `testing` | `from testing import ...` | Testing y aserciones |

## Convenciones

- Todos los módulos se importan explícitamente: `from math import math`
- Las funciones se llaman con namespace: `math.max(a, b)`
- snake_case para todo: funciones, métodos, tipos
- `T` mayúscula = type parameter (genéricos)
