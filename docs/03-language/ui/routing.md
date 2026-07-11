# Routing y Navegación

**Status:** Draft v2.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Transiciones

---

## 1. Auto-Routing

Cada vista declara sus propias rutas con `view()`. El `router` padre detecta automáticamente estas declaraciones.

### 1.1 Vista básica

```kyx
# home.kyx
view("/")
<view>
    <text value="Inicio" />
</view>
```

```kyx
# login.kyx
view("/login")
<view>
    <text_field bind=@email />
    <password_field bind=@password />
    <button text="Ingresar" click=@login />
</view>
```

### 1.2 Múltiples rutas por vista

```kyx
# home.kyx
view("/", "/home", "/index")

<view>
    <text value="Inicio" />
</view>
```

```kyx
# profile.kyx
view("/profile", "/me")

<view>
    <text value="Perfil" />
</view>
```

El primer path es el **canónico** (usado internamente para navegación). Los siguientes son **alias**.

### 1.3 App entry point

El `router` solo necesita los componentes como hijos — detecta sus `view()` automáticamente:

```kyx
# app.kyx
<app>
    <router>
        <home_view />
        <login_view />
        <dashboard_view />
        <user_detail />
    </router>
</app>
```

Sin `<route>` redundante. Cada componente aporta su ruta desde su propio archivo.

### 1.4 Cómo funciona

1. El compilador lee `view("/path")` en cada `.kyx`
2. Genera un registro de rutas: `{"/path" → component_id}`
3. El `<router>` usa ese registro en runtime para matchear la URL actual
4. `view("/", "/home")` genera: `{"/" → component_id, "/home" → component_id}`

---

## 2. Parámetros de Ruta

### 2.1 `{param}` en el path

```kyx
# user_detail.kyx
view("/users/{id}")

@(
    params: {str: str} = route_params()
    id: str = params.get("id") ?? ""
    user: User? = fetch_user(id)
)
<view>
    <text value="Usuario: " + user.name />
</view>
```

`route_params()` es una función built-in del router. Devuelve un `{str: str}` con los parámetros extraídos de la URL.

### 2.2 Múltiples parámetros

```kyx
view("/posts/{year}/{slug}")
@(
    params = route_params()
    year: str = params.get("year") ?? ""
    slug: str = params.get("slug") ?? ""
)
```

### 2.3 Parámetros opcionales

```kyx
view("/search/{query?}")
# /search/foo → params.query = "foo"
# /search     → params.query = None
```

---

## 3. Router con Configuración

El `<route>` se usa **solo** cuando necesitas configuración extra (guards, lazy loading, anidamiento, opciones).

### 3.1 Sin `<route>` — auto-routing puro

```kyx
<router>
    <home_view />
    <login_view />
</router>
```

Las rutas se infieren de `view()` en cada componente.

### 3.2 Con `<route>` — configuración explícita

```kyx
<router lazy=true>
    <route path="/dashboard" component=@dashboard_view guards=@auth_guard />
    <route path="/admin" component=@admin_view lazy=true />
    <route path="*" component=@not_found_view />
</router>
```

Usa `<route>` cuando necesites:
- `guards` — protectores de ruta
- `lazy` — lazy loading explícito (o heredado del router)
- `path="*"` — catch-all / 404
- `redirect` — redirección
- Sobrescribir el path declarado en `view()` de un componente

### 3.3 Rutas anidadas

```kyx
<router>
    <route path="/admin" guards=@admin_only>
        <route path="/users" component=@admin_users />
        <route path="/settings" component=@admin_settings />
    </route>
</router>
```

### 3.4 Mix: auto-routing + <route>

```kyx
<router>
    # auto-routing: infiere rutas de view()
    <home_view />
    <login_view />

    # explícito: necesita configuración extra
    <route path="/dashboard" component=@dashboard_view guards=@auth />
    <route path="*" component=@not_found_view />
</router>
```

---

## 4. Navegación

### 4.1 Navegación programática

```kyle
fn go_to_login():
    navigate("/login")

fn go_to_user(id: i32):
    navigate("/users/" + id.to_str())

fn go_back():
    navigate_back()

fn replace_current(path: str):
    navigate_replace(path)  # sin agregar al historial
```

### 4.2 Links

```kyx
<link to="/dashboard">
    <text value="Ir al Dashboard" />
</link>

<link to="/users/42" tpl=Primary>
    <text value="Ver Usuario 42" />
</link>

<link to="https://externo.com" external=true>
    <text value="Sitio externo" />
</link>
```

### 4.3 Navegación con estado

```kyle
fn go_to_edit(id: i32):
    navigate("/edit/" + id.to_str(), state: {
        "from": "list",
        "scroll_position": scroll_y,
    })
```

### 4.4 Navegación por alias

Si un componente declaró `view("/profile", "/me")`:

```kyle
navigate("/me")      # funciona
navigate("/profile") # misma vista, path canónico
```

---

## 5. Guardias

### 5.1 Protección de rutas

```kyx
@(
    fn on_before_enter() RouteResult:
        if !is_authenticated():
            RouteResult.Redirect("/login")
        else:
            RouteResult.Continue
)

<route
    path="/dashboard"
    component=@dashboard_view
    before_enter=@on_before_enter
/>
```

### 5.2 Tipos de guardia

| Guardia | Momento | Uso |
|---------|---------|-----|
| `before_enter` | Antes de entrar | Auth checks |
| `before_leave` | Antes de salir | Confirmar cambios no guardados |
| `on_error` | Error al cargar | Error boundary |

```kyx
@(
    fn confirm_leave() RouteResult:
        if has_unsaved_changes():
            RouteResult.Confirm("¿Descartar cambios?")
        else:
            RouteResult.Continue
)

<route
    path="/edit"
    component=@edit_form
    before_leave=@confirm_leave
/>
```

### 5.3 Guardia inline en auto-routing

```kyx
# dashboard.kyx
view("/dashboard")
@(
    fn on_before_enter() RouteResult:
        if !is_authenticated():
            RouteResult.Redirect("/login")
        RouteResult.Continue
)
<view>
    <text value="Dashboard" />
</view>
```

Si no hay `<route>`, el router busca funciones `on_before_enter`, `on_before_leave`, `on_error` en el `@()` block del componente y las usa automáticamente.

---

## 6. Lazy Loading

```kyx
# Todo lazy
<router lazy=true>
    <home_view />       # carga bajo demanda
    <dashboard_view />  # carga bajo demanda
</router>

# Mix
<router>
    <home_view />           # eager
    <route path="/heavy" component=@heavy_view lazy=true />  # lazy
</router>
```

Web target → code splitting automático:

```javascript
// Generado automáticamente
const routes = {
    '/dashboard': () => import('./pages/dashboard.wasm'),
    '/heavy': () => import('./pages/heavy.wasm'),
};
```

---

## 7. Transiciones

```kyx
<router transition=PageSlide transition_duration=300>
    <home_view />
    <login_view />
    <dashboard_view />
</router>
```

Ver [animation.md](animation.md) para tipos de transición.

---

## 8. URL y Estado del Browser

### 8.1 Web target

- History API (`pushState`, `popState`)
- URLs limpias (sin `#`)

```javascript
window.addEventListener('popstate', (event) => {
    wasm.exports.onNavigate(event.state?.path || '/');
});

function navigate(path) {
    window.history.pushState({ path }, '', path);
    wasm.exports.onNavigate(path);
}
```

### 8.2 Desktop target

- Sin URL de browser
- Stack interno de navegación
- `navigate_back()` popea del stack

---

## 9. Deep Linking

```kyle
fn on_app_start():
    initial_path = get_initial_url()  # web: window.location.pathname
    navigate(initial_path)
```

---

## 10. Resumen: Auto-Routing vs <route>

| Situación | Usar |
|-----------|------|
| Vista normal, una ruta | `view("/path")` + componente hijo del router |
| Vista con alias | `view("/path", "/alias")` |
| Guardias | `on_before_enter` en `@()` del componente, o `<route guards=...>` |
| Lazy loading | `router lazy=true` o `<route lazy=true>` |
| Rutas anidadas | `<route path="/admin">` con hijos `<route>` |
| Catch-all / 404 | `<route path="*" component=@not_found_view />` |
| Config extra (redirect, title, etc.) | `<route>` |
| Sin configuración extra | Auto-routing puro — solo `view()` |

> **Regla de oro:** Si no necesitas guards, lazy loading, o anidamiento, no escribas `<route>`. El auto-routing es suficiente.

---

## 11. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Page transitions
