# Project Layout

> Estructura recomendada para proyectos Kyle.

## App project

```
myapp/
├── ky.toml              # Configuración del proyecto
├── src/
│   ├── main.ky          # Punto de entrada
│   ├── lib.ky           # Librería del proyecto
│   └── utils.ky         # Módulos auxiliares
├── tests/
│   └── test_main.ky     # Tests
└── target/
    ├── debug/           # Build artifacts (debug)
    └── release/         # Build artifacts (release)
```

## API project

```
myapi/
├── ky.toml
├── src/
│   ├── main.ky
│   ├── routes.ky
│   └── models.ky
├── tests/
└── target/
```

## Bare script

```
script.ky                 # Archivo único, sin ky.toml
```

## ky.toml

```toml
name = "myapp"
version = "0.1.0"
edition = "2024"
```

## Ver también

- `build.md` — Compilación de proyectos
- `first-program.md` — Crear primer proyecto
