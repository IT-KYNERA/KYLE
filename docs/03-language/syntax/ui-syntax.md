# Kyle UI Syntax Specification

**Versión:** 1.0  
**Estado:** Especificación

---

## 1. Filosofía

Kyle UI es un sistema de UI declarativo construido sobre el lenguaje Kyle.

Un archivo `.kyx` representa una vista.

- Si contiene `page(...)`, representa una página.
- Si no contiene `page(...)`, representa un componente reutilizable.

No existe JavaScript. Todo el código Kyle se integra mediante expresiones `@`.

---

## 2. Archivos

```
Login.ky        ← Lógica opcional
Login.kyx       ← Vista

Button.kyx      ← Componente
```

El archivo `.ky` es opcional. Toda la lógica puede ir dentro del `.kyx`.

---

## 3. Página

```kyle
page("/login")

<view>
    <text value="Login" />
</view>
```

---

## 4. Componente

```kyle
<view>
    <text value="Hello" />
</view>
```

Sin `page(...)` → es un componente reutilizable.

---

## 5. Código Kyle

Fragmentos pequeños con `@`:

```kyx
<text value=@user.name />
```

Varias líneas con `@(...)`:

```kyle
@(
    email: str
    password: str

    fn login():
        ...
)
```

---

## 6. Variables

```kyle
@total = products.count

<text value=@total />
```

---

## 7. Condicionales

```kyx
@if(user.isAdmin):
    <button text="Delete" />
```

```kyx
@if(user.isAdmin):
    <admin_panel />
@else:
    <user_panel />
```

---

## 8. Match

```kyx
@match(state):
    Loading:
        <spinner />
    Success:
        <dashboard />
    Error:
        <error_view />
```

---

## 9. For

```kyx
@for(product in products):
    <product_card product=@product />
```

---

## 10. Eventos

```kyx
<button click=@login />

<text_field change=@on_change />

<text_field input=@on_input />
```

---

## 11. Binding

```kyx
<text_field bind=@email />
```

---

## 12. Componentes hijos

```kyx
<card>
    <text />
    <button />
</card>
```

---

## 13. Slots

Declaración en el componente:

```kyle
slot header
slot Content
```

Uso:

```kyx
<card>
    <header>
        <text value="Title" />
    </header>
    <content>
        <text value="Body" />
    </content>
</card>
```

Las páginas no admiten slots.

---

## 14. Ciclo de vida

```kyle
fn on_create(): ...
fn on_init(): ...
fn on_ready(): ...
fn on_render(): ...
fn on_rendered(): ...
fn on_dispose(): ...
```

---

## 15. Componentes nativos

```
App, View, Column, Row, Grid, Stack, Spacer, Container,
Card, Surface, Scroll, List, Form,
Text, Label, Button, IconButton, Image, Icon, Divider,
TextField, PasswordField, TextArea, Checkbox, Radio, Switch, Slider,
Progress, Spinner, Select, Menu, MenuItem,
Toolbar, AppBar, BottomBar, Navigation, Tab, Tabs,
Dialog, Drawer, Popup, Snackbar, Tooltip,
Canvas, Video, Audio, Map, Chart, WebView
```

---

## 16. Recursos (Style, Layout, Typography, Animation)

```kyle
style<button> Primary:
    background = theme.primary
    color = white
    radius = md
    padding = (12, 24)

layout<Column> Center:
    align = center
    justify = center
    spacing = 20
```

Uso:

```kyx
<button style=Primary />
<Column layout=Center />
```

---

## 17. Template

```kyle
tpl<button> Primary:
    style = primary_style
    animation = primary_animation
    cursor = pointer
    ripple = true
```

Uso:

```kyx
<button tpl=Primary />
```

---

## 18. Theme

```kyle
theme Light:
    primary = #0066FF
    background = white
    text = black
```

Uso:

```kyx
<App theme=Light />
```

---

## 19. Ejemplo completo: Login

```kyle
page("/login")

@(
    email: str
    password: str

    fn login():
        print("login with " + email)
)

<view>
    <Column layout=Center>
        <text value="Login" typography=Title />
        <text_field bind=@email />
        <password_field bind=@password />
        <button tpl=Primary text="Ingresar" click=@login />
    </Column>
</view>
```
