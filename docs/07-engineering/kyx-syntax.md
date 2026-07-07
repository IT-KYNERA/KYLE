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

<View>
    <Text value="Login" />
</View>
```

---

## 4. Componente

```kyle
<View>
    <Text value="Hello" />
</View>
```

Sin `page(...)` → es un componente reutilizable.

---

## 5. Código Kyle

Fragmentos pequeños con `@`:

```xml
<Text value=@user.name />
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

<Text value=@total />
```

---

## 7. Condicionales

```xml
@if(user.isAdmin):
    <Button text="Delete" />
```

```xml
@if(user.isAdmin):
    <AdminPanel />
@else:
    <UserPanel />
```

---

## 8. Match

```xml
@match(state):
    Loading:
        <Spinner />
    Success:
        <Dashboard />
    Error:
        <ErrorView />
```

---

## 9. For

```xml
@for(product in products):
    <ProductCard product=@product />
```

---

## 10. Eventos

```xml
<Button click=@login />

<TextField change=@onChange />

<TextField input=@onInput />
```

---

## 11. Binding

```xml
<TextField bind=@email />
```

---

## 12. Componentes hijos

```xml
<Card>
    <Text />
    <Button />
</Card>
```

---

## 13. Slots

Declaración en el componente:

```kyle
slot header
slot Content
```

Uso:

```xml
<Card>
    <header>
        <Text value="Title" />
    </header>
    <Content>
        <Text value="Body" />
    </Content>
</Card>
```

Las páginas no admiten slots.

---

## 14. Ciclo de vida

```kyle
fn onCreate(): ...
fn onInit(): ...
fn onReady(): ...
fn onRender(): ...
fn onRendered(): ...
fn onDispose(): ...
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
style<Button> Primary:
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

```xml
<Button style=Primary />
<Column layout=Center />
```

---

## 17. Template

```kyle
tpl<Button> Primary:
    style = PrimaryStyle
    animation = PrimaryAnimation
    cursor = pointer
    ripple = true
```

Uso:

```xml
<Button tpl=Primary />
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

```xml
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

<View>
    <Column layout=Center>
        <Text value="Login" typography=Title />
        <TextField bind=@email />
        <PasswordField bind=@password />
        <Button tpl=Primary text="Ingresar" click=@login />
    </Column>
</View>
```
