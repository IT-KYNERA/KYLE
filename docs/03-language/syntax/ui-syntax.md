# Kyle UI Syntax Specification

**Version:** 1.0 
**Status:** Specification

> **Arquitectura e implementaci&oacute;n:** Ver [`docs/10-design/rfc/0002-ui-architecture.md`](../../10-design/rfc/0002-ui-architecture.md) para el roadmap t&eacute;cnico completo (WASM, multiplataforma, custom OS).  
> **Traducci&oacute;n multi-target:** Ver [`docs/10-design/rfc/0003-ui-translation.md`](../../10-design/rfc/0003-ui-translation.md) para la arquitectura de traducci&oacute;n a JS/nativo/WASM.  
> **Sistema de estilos:** Ver [`docs/03-language/ui/style-system.md`](../../03-language/ui/style-system.md) para el sistema de estilos tipado (sin CSS).  
> **Estado y eventos:** Ver [`docs/03-language/ui/state-events.md`](../../03-language/ui/state-events.md) para estado, binding y eventos.  
> **Animaciones:** Ver [`docs/03-language/ui/animation.md`](../../03-language/ui/animation.md) para animaciones y transiciones.  
> **Routing:** Ver [`docs/03-language/ui/routing.md`](../../03-language/ui/routing.md) para navegaci&oacute;n y rutas.  
> **Accesibilidad:** Ver [`docs/03-language/ui/accessibility.md`](../../03-language/ui/accessibility.md) para a11y.  
> **Distribuci&oacute;n:** Ver [`docs/07-tools/distribution.md`](../../07-tools/distribution.md) para el plan de instalaci&oacute;n multi-plataforma.

---

## 1. Philosophy

Kyle UI is un sistema de UI declarativo construido about language Kyle.

Un file `.kyx` represents una vista.

- Si conhas `view(...)`, represents una page (ruteable).
- Si no conhas `view(...)`, represents un componente reutilizable.

No existe JavaScript. Todo code Kyle se integra using expresionis `@`.

---

## 2. Files

```
Login.ky ← Logic opcional
Login.kyx ← Vista

Button.kyx ← Componente
```

El file `.ky` is opcional. Toda logic can ir inside del `.kyx`.

---

## 3. Page

```kyle
view("/login")

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

Sin `view(...)` → is un componente reutilizable.

---

## 5. Code Kyle

Fragmentos pequenos with `@`:

```kyx
<text value=@user.name />
```

Varias lines with `@(...)`:

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

## 12. Componentis hijos

```kyx
<card>
 <text />
 <button />
</card>
```

---

## 13. Slots

Declaration en componente:

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

Las views (con `view(...)`) no admiten slots.

---

## 14. Ciclo de vida

```kyle
fn on_created(this):
    # Se llama una vez, después del constructor
    # Inicializar recursos

fn on_mounted(this):
    # Se llama cuando el componente se monta en el DOM/padre
    # Empezar timers, fetch data, etc.

fn on_updated(this, changed: {str}):
    # Se llama cuando cambian las props
    # changed = conjunto de props que cambiaron

fn on_unmounted(this):
    # Se llama cuando el componente se desmonta
    # Cleanup: timers, listeners, etc.

fn on_error(this, error: str):
    # Error boundary: captura errores en componentes hijos
```

---

## 15. Componentis nativos

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

## 19. Example completo: Login

```kyle
view("/login")

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
