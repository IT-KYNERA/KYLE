# Estado y Eventos

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [style-system.md](style-system.md) — Sistema de estilos
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target

---

## 1. Estado de Componente

### 1.1 Estado local

Cada componente puede tener estado local. Se declara como variables `^T` (mutables)
dentro del bloque `@(...)` del .kyx:

```kyx
@(
    count: ^i32 = 0
    fn increment():
        count = count + 1
)
<view>
    <text value="Contador: " + count.to_str() />
    <button text="+" click=@increment />
</view>
```

### 1.2 Estado derivado

Estado que se calcula automáticamente a partir de otras variables:

```kyx
@(
    items: ^{str} = {}
    filtered: ^{str} = {}  # se actualiza cuando cambia items o filter_text
    filter_text: ^str = ""

    fn update_filter():
        filtered = items.filter(fn(x): x.contains(filter_text))
)
```

### 1.3 Props (parámetros del componente)

Los props se pasan como atributos desde el padre:

```kyx
<!-- Uso -->
<user_card name="Juan" age=30 />
```

```kyx
<!-- Definición en UserCard.kyx -->
@(
    name: str       # prop requerido
    age: i32 = 0    # prop opcional con default
    on_click: ^&(fn ())  # callback
)
<view>
    <text value=name />
    <text value=age.to_str() />
</view>
```

### 1.4 Estado global (Context)

Para estado compartido entre componentes no relacionados:

```kyx
@(
    # Definir contexto
    context AuthContext:
        user: User?
        token: str?
        fn login(email: str, password: str):
            # ...
)
```

Consumir contexto:

```kyx
@(
    auth = use_context(AuthContext)
)
<view>
    @if auth.user != None:
        <text value="Bienvenido, " + auth.user.name />
    @else:
        <button text="Login" click=@show_login />
</view>
```

---

## 2. Binding Bidireccional

El binding sincroniza automáticamente el estado con la UI.

### 2.1 One-way binding (estado → UI)

```kyx
<text value=@name />           # se actualiza cuando name cambia
```

### 2.2 Two-way binding (estado ↔ UI)

```kyx
<text_field bind=@email />      # email se actualiza al escribir, y viceversa
<checkbox bind=@accept_terms />
```

### 2.3 Binding a props de componentes

```kyx
<child_component prop=@parent_var />
```

Cuando `parent_var` cambia, `prop` se actualiza automáticamente.

### 2.4 Cómo funciona el binding

El compilador transforma el binding en:

Web target:
```javascript
// Generado automáticamente
const state = { email: '' };
const input = document.createElement('input');

// One-way: estado → UI
function updateUI() {
    input.value = state.email;
}

// Two-way: UI → estado
input.addEventListener('input', () => {
    state.email = input.value;
    // Notificar a WASM del cambio
    wasm.exports.onStateChange('email', state.email);
});
```

---

## 3. Eventos

### 3.1 Eventos de UI

```kyx
<button click=@handle_click />
<text_field change=@handle_change />
<text_field input=@on_input />
<checkbox change=@on_toggle />
<select change=@on_select />
<form submit=@handle_submit />
<view mouse_enter=@on_hover />
<view mouse_leave=@on_leave />
<view focus=@on_focus />
<view blur=@on_blur />
<view keydown=@on_keypress />
<view scroll=@on_scroll />
```

### 3.2 Parámetros de evento

```kyx
@(
    fn handle_click(event: MouseEvent):
        print("click en: " + event.x.to_str() + ", " + event.y.to_str())

    fn handle_keypress(event: KeyboardEvent):
        if event.key == "Enter":
            submit()
)
```

### 3.3 Eventos personalizados

Los componentes pueden emitir eventos personalizados:

```kyx
<!-- Definición -->
@(
    on_save: ^&(fn (data: str))
    data: ^str = ""

    fn save():
        on_save(data)
)
<view>
    <text_field bind=@data />
    <button text="Guardar" click=@save />
</view>
```

```kyx
<!-- Uso -->
<my_form on_save=@handle_save />
```

### 3.4 Event bubbling

Por defecto, los eventos burbujean hacia el padre. Se puede detener:

```kyx
@(
    fn handle_click(event: MouseEvent):
        event.stop_propagation()
        # solo este componente maneja el click
)
```

---

## 4. Reactividad

### 4.1 Cómo se actualiza la UI

Cuando una variable de estado cambia:

```
count = count + 1
    │
    ▼
Runtime detecta el cambio
    │
    ▼
Busca qué componentes dependen de count
    │
    ▼
Re-renderiza solo esos componentes
    │
    ▼
Genera nuevas llamadas a createElement / draw
```

### 4.2 Implementación por target

**Web:** El runtime JS detecta cambios vía Proxy o setters:
```javascript
// Generado automáticamente
function createState(initial) {
    const state = reactive({ ...initial });
    const watchers = new Map();

    state.$watch = (key, fn) => {
        if (!watchers.has(key)) watchers.set(key, []);
        watchers.get(key).push(fn);
    };

    return new Proxy(state, {
        set(target, key, value) {
            target[key] = value;
            // Notificar a watchers
            if (watchers.has(key)) {
                for (const fn of watchers.get(key)) fn(value);
            }
            return true;
        }
    });
}
```

**Desktop:** El runtime Kyle maneja reactive updates vía diff del árbol de componentes.

### 4.3 Optimización

Solo los componentes afectados se re-renderizan. El framework mantiene un
grafo de dependencias:

```
count ───→ CounterView (depende de count)
           └──→ Button (no depende de count → skip)
```

---

## 5. Comunicación entre Componentes

### 5.1 Padre → Hijo (props)

```kyx
<child_component
    title="Mi Título"
    items=@data_list
/>
```

### 5.2 Hijo → Padre (eventos/callbacks)

```kyx
<child_component
    on_delete=@handle_delete
    on_select=@handle_select
/>
```

### 5.3 Entre hermanos (vía padre)

```kyx
@(
    selected_id: ^i32 = -1

    fn on_select(id: i32):
        selected_id = id
)
<view>
    <item_list
        items=@items
        on_select=@on_select
    />
    <item_detail
        item_id=@selected_id
    />
</view>
```

### 5.4 Entre componentes no relacionados (vía context)

```kyx
@(
    context AppState:
        theme: Theme = LightTheme
        user: User?
        notifications: {Notification}

    # Cualquier componente puede acceder a AppState
    state = use_context(AppState)
)
```

---

## 6. Ciclo de Vida de Eventos

```
Usuario hace click
    │
    ▼
Evento nativo (click del browser / touch)
    │
    ▼
[Web] addEventListener captura
[Desktop] Event loop captura
    │
    ▼
Traduce a evento Kyle (MouseEvent, KeyboardEvent, etc.)
    │
    ▼
Ejecuta el callback Kyle (en WASM o nativo)
    │
    ▼
Callback modifica estado
    │
    ▼
Runtime detecta cambios → re-renderiza
    │
    ▼
[Web] Actualiza DOM
[Desktop] Redibuja con Skia
```

---

## 7. Formularios

### 7.1 Form básico

```kyx
@(
    email: ^str = ""
    password: ^str = ""
    errors: ^{str} = {}
    loading: ^bool = false

    fn validate() bool:
        errors = {}
        if email == "":
            errors.push("Email requerido")
        if password == "":
            errors.push("Contraseña requerida")
        if password.len() < 6:
            errors.push("Mínimo 6 caracteres")
        errors.len() == 0

    fn handle_submit():
        if !validate():
            return
        loading = true
        # login...
)

<form submit=@handle_submit>
    <text_field
        label="Email"
        bind=@email
        error=if errors.contains("Email requerido"): "Requerido" else: ""
    />
    <password_field
        label="Contraseña"
        bind=@password
        error=if errors.contains("Contraseña requerida"): "Requerido" else: ""
    />
    <button
        tpl=Primary
        text=if loading: "Cargando..." else: "Ingresar"
        disabled=@loading
    />
</form>
```

### 7.2 Validación tipada

```kyle
final class ValidationRule<T>:
    fn validate(this, value: T) str?  # None = ok, Some(msg) = error

# Rules predefinidas
fn required<T>() ValidationRule<T>:
    ValidationRule(fn(val: T) str?:
        if val == "" or val == None:
            "Este campo es requerido"
        None
    )

fn min_length(n: i32) ValidationRule<str>:
    ValidationRule(fn(val: str) str?:
        if val.len() < n:
            "Mínimo " + n.to_str() + " caracteres"
        None
    )

fn email() ValidationRule<str>:
    ValidationRule(fn(val: str) str?:
        if !val.contains("@"):
            "Email inválido"
        None
    )
```

---

## 8. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [style-system.md](style-system.md) — Sistema de estilos
- [animation.md](animation.md) — Animaciones
- [routing.md](routing.md) — Routing/Navegación
