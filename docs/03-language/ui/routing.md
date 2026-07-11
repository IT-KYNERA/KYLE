# Routing y Navegación

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos

---

## 1. Definición de Rutas

### 1.1 Páginas

Cada página se define con `page()` en su .kyx:

**login.kyx:**
```kyx
page("/login")

<view>
    <text value="Iniciar Sesión" />
    <text_field bind=@email />
    <password_field bind=@password />
    <button text="Ingresar" click=@login />
</view>
```

**dashboard.kyx:**
```kyx
page("/dashboard")

<view>
    <appbar title="Dashboard" />
    <text value="Bienvenido" />
</view>
```

### 1.2 Router

El router se declara en el entry point de la app:

```kyx
# app.kyx
page("/")

<app>
    <router>
        <route path="/" component=@home_view />
        <route path="/login" component=@login_view />
        <route path="/dashboard" component=@dashboard_view />
        <route path="/users/:id" component=@user_detail />
        <route path="/settings" component=@settings_view />
        <route path="*" component=@not_found_view />
    </router>
</app>
```

### 1.3 Rutas con parámetros

```kyx
page("/users/:id")
@(
    id: str = route_params().get("id") ?? ""
    user: User? = fetch_user(id)
)
<view>
    <text value="Usuario: " + user.name />
</view>
```

### 1.4 Rutas anidadas

```kyx
<router>
    <route path="/admin">
        <route path="/users" component=@admin_users />
        <route path="/settings" component=@admin_settings />
    </route>
</router>
```

---

## 2. Navegación

### 2.1 Navegación programática

```kyle
# Dentro de un componente
fn go_to_login():
    navigate("/login")

fn go_to_user(id: i32):
    navigate("/users/" + id.to_str())

fn go_back():
    navigate_back()

fn replace_current(path: str):
    navigate_replace(path)  # sin agregar al historial
```

### 2.2 Links

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

### 2.3 Navegación con estado

```kyle
fn go_to_edit(id: i32):
    navigate("/edit/" + id.to_str(), state: {
        "from": "list",
        "scroll_position": scroll_y,
    })
```

---

## 3. Guardias de Ruta

### 3.1 Protección de rutas

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

### 3.2 Tipos de guardia

| Guardia | Momento | Uso |
|---------|---------|-----|
| `before_enter` | Antes de entrar a la ruta | Auth checks |
| `before_leave` | Antes de salir de la ruta | Confirmar cambios no guardados |
| `on_error` | Si hay error al cargar la ruta | Error boundary |

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

---

## 4. Lazy Loading

Las páginas se cargan bajo demanda:

```kyx
<router lazy=true>
    # dashboard_view solo se carga cuando se navega a /dashboard
    <route path="/dashboard" component=@dashboard_view />
    <route path="/settings" component=@settings_view />
</router>
```

Web target → code splitting automático:
```javascript
// Generado automáticamente
const routes = {
    '/dashboard': () => import('./pages/dashboard.wasm'),
    '/settings': () => import('./pages/settings.wasm'),
};
```

---

## 5. Transiciones de Página

```kyx
<router
    transition=PageSlide
    transition_duration=300
>
    <route path="/" component=@home_view />
    <route path="/details" component=@details_view />
</router>
```

Ver [animation.md](animation.md) para tipos de transición.

---

## 6. URL y Estado del Browser

### 6.1 Web target

- Usa History API (`pushState`, `popState`)
- URLs limpias (sin `#`)
- SSR compatible (server-side rendering)

```javascript
// Generado automáticamente por el compilador
window.addEventListener('popstate', (event) => {
    wasm.exports.onNavigate(event.state?.path || '/');
});

function navigate(path) {
    window.history.pushState({ path }, '', path);
    wasm.exports.onNavigate(path);
}
```

### 6.2 Desktop target

- No hay URL de browser
- El router mantiene un stack interno de navegación
- `navigate_back()` popea del stack

---

## 7. Deep Linking

```kyle
# Al iniciar la app, el router procesa la URL inicial
fn on_app_start():
    initial_path = get_initial_url()  # web: window.location.pathname
    navigate(initial_path)
```

---

## 8. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Page transitions
