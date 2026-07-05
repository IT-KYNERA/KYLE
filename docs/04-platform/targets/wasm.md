# WASM — WebAssembly Target

**Versión:** 1.0  
**Estado:** Especificación

---

## 1. ¿Qué es?

Kyle compila a WebAssembly via LLVM (`wasm32-unknown-unknown`).
Esto permite ejecutar código Kyle en el navegador.

```
.ky → LLVM IR → wasm32 → .wasm → navegador
```

---

## 2. Compilación

```bash
# Compilar a WASM
ky build --target wasm32-unknown-unknown app.ky

# Optimizado para tamaño
ky build --target wasm32-unknown-unknown -O3 app.ky
```

### Limitaciones del target WASM

| Recurso | Disponible |
|---------|-----------|
| CPU (cálculos) | ✅ |
| Memoria lineal | ✅ |
| Math (f32/f64) | ✅ |
| Strings | ✅ |
| File system | ❌ (no hay FS en WASM) |
| Networking | ❌ (no hay sockets) |
| Threads | ❌ |
| Console/print | ❌ (requiere JS glue) |

---

## 3. web — Browser API bindings

El package `web` expone APIs del navegador a Kyle compilado a WASM.

```kyle
from web import document, console, fetch
```

### DOM

```kyle
from web import document

div = document.getElementById("app")
div.textContent = "Hola desde Kyle!"

# Crear elementos
btn = document.createElement("button")
btn.textContent = "Click me"
btn.onclick = (event):
    console.log("clicked!")
div.appendChild(btn)
```

### Fetch (HTTP desde el browser)

```kyle
from web import fetch, Response

res = fetch("/api/users")
if res.ok:
    users = res.json()
    for user in users:
        print(user["name"])
```

### Console

```kyle
from web import console

console.log("hello from Kyle")
console.error("something went wrong")
```

### Canvas 2D

```kyle
from web import document, canvas

canvas = document.getElementById("game")
ctx = canvas.getContext("2d")

ctx.fillStyle = "red"
ctx.fillRect(10, 10, 100, 50)

ctx.fillStyle = "blue"
ctx.font = "20px sans-serif"
ctx.fillText("Kyle!", 20, 80)
```

---

## 4. Estructura del package web

```
packages/web/
├── ky.toml
└── src/
    ├── lib.ky          # Exportaciones principales
    ├── dom.ky          # DOM API (getElementById, createElement, etc.)
    ├── fetch.ky        # Fetch API
    ├── canvas.ky       # Canvas 2D
    ├── console.ky      # Console API
    └── events.ky       # Event listeners
```

### Implementación

Las APIs del navegador se exponen via `extern fn` con `@link "js"`:

```kyle
@link "js"

extern fn js_getElementById(id: ptr) ptr
extern fn js_setTextContent(el: ptr, text: ptr)
extern fn js_createElement(tag: ptr) ptr
extern fn js_appendChild(parent: ptr, child: ptr)
extern fn js_fetch(url: ptr) ptr
```

Un runtime de JS glue traduce las llamadas a las APIs reales del navegador.

---

## 5. Flujo de trabajo

```bash
# 1. Compilar Kyle a WASM
ky build --target wasm32-unknown-unknown app.ky -o app.wasm

# 2. Crear HTML con JS glue
cat > index.html << 'HTML'
<!DOCTYPE html>
<script src="runtime.js"></script>
<script>
    const wasm = await WebAssembly.instantiateStreaming(
        fetch("app.wasm"), { js: KyJsGlue }
    );
    wasm.instance.exports.main();
</script>
HTML
```

---

## 6. Plan de implementación

| Fase | Descripción | Estado |
|------|-------------|--------|
| 1 | Compilación WASM vía LLVM | ✅ (LLVM target existe) |
| 2 | JS glue runtime básico | 🔜 |
| 3 | web: console + DOM básico | 🔜 |
| 4 | web: fetch, canvas, events | 🔜 |
| 5 | Optimización de tamaño WASM | 🔜 |
| 6 | Reactive UI framework (JSX-like) | 📅 |
