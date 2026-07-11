# Sistema de Estilos Tipado

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target

---

## 1. Filosofía

Kyle NO usa CSS. Los estilos son **tipos Kyle** con propiedades fuertemente tipadas,
verificadas en tiempo de compilación. No hay strings mágicos, no hay selectores CSS,
no hay hojas de estilo externas.

| Aspecto | CSS | Kyle Style |
|---------|:---:|:-----------:|
| Sintaxis | `.clase { prop: val }` | `style<comp> Name: prop = val` |
| Tipado | ❌ Strings | ✅ Tipos Kyle verificados |
| Scope | Global (cascade) | Local al componente |
| Compilación | Texto | AST → target nativo |
| Variables | `--var` (custom props) | `theme.Name` (tipado) |
| Responsive | `@media` queries | `@media(prop: val)` tipado |

---

## 2. Tipos de Estilo

### 2.1 Color

```kyle
final class Color:
    r: u8
    g: u8
    b: u8
    a: f32 = 1.0

    # Constructores
    static fn from_hex(hex: str) Color
    static fn from_rgba(r: u8, g: u8, b: u8, a: f32) Color
    static fn from_hsl(h: f32, s: f32, l: f32) Color

    # Colores predefinidos
    static fn transparent() Color
    static fn black() Color
    static fn white() Color
    static fn primary() Color      # del theme activo
    static fn accent() Color

    # Modificación
    fn darken(this, amount: f32) Color    # 0.0 = no change, 1.0 = black
    fn lighten(this, amount: f32) Color   # 0.0 = no change, 1.0 = white
    fn with_alpha(this, a: f32) Color     # nueva alpha
```

### 2.2 Spacing

```kyle
final class Spacing:
    top: f32
    right: f32
    bottom: f32
    left: f32

    static fn all(value: f32) Spacing
    static fn symmetric(horizontal: f32, vertical: f32) Spacing
    static fn only(top: f32, right: f32, bottom: f32, left: f32) Spacing
```

### 2.3 Length

```kyle
enum Length:
    Px(f32)         # píxeles
    Percent(f32)    # porcentaje del contenedor (0.0-100.0)
    Auto            # automático
    Fill            # llena el espacio disponible
    Vw(f32)         # viewport width %
    Vh(f32)         # viewport height %
```

### 2.4 Border

```kyle
final class Border:
    width: f32
    color: Color
    style: BorderStyle  # Solid, Dashed, Dotted, None

final class Shadow:
    x: f32            # offset X
    y: f32            # offset Y
    blur: f32         # blur radius
    spread: f32       # spread radius
    color: Color
```

### 2.5 Tipografía

```kyle
enum FontWeight:
    Thin      # 100
    Light     # 300
    Normal    # 400
    Medium    # 500
    SemiBold  # 600
    Bold      # 700
    Black     # 900

enum TextAlign:
    Left
    Center
    Right
    Justify

final class TextStyle:
    font_family: str
    font_size: f32
    font_weight: FontWeight
    line_height: f32
    letter_spacing: f32
    color: Color
    align: TextAlign
    decoration: TextDecoration  # None, Underline, LineThrough
```

### 2.6 Layout

```kyle
enum Display:
    Flex
    Grid
    None

enum FlexDirection:
    Row
    Column
    RowReverse
    ColumnReverse

enum Alignment:
    Start
    Center
    End
    Stretch
    Baseline

enum Overflow:
    Visible
    Hidden
    Scroll
    Auto

enum Position:
    Static
    Relative
    Absolute(position: AbsolutePosition)
    Fixed(position: AbsolutePosition)
    Sticky

final class AbsolutePosition:
    top: Length
    left: Length
    right: Length
    bottom: Length
```

### 2.7 Transform

```kyle
final class Transform:
    translate_x: f32
    translate_y: f32
    scale_x: f32 = 1.0
    scale_y: f32 = 1.0
    rotate: f32        # grados
    skew_x: f32
    skew_y: f32

final class Transition:
    property: str     # "opacity", "transform", "background", "all"
    duration: i32     # milisegundos
    easing: Easing    # Ease, Linear, EaseIn, EaseOut, EaseInOut, CubicBezier(f1,f2,f3,f4)
    delay: i32        # milisegundos
```

### 2.8 Style class completo

```kyle
final class Style:
    # Background
    background: Color?
    background_image: str?        # URL
    background_repeat: bool?
    background_size: Length?

    # Texto
    color: Color?
    font: TextStyle?

    # Spacing
    padding: Spacing?
    margin: Spacing?

    # Border
    border: Border?
    border_top: Border?
    border_right: Border?
    border_bottom: Border?
    border_left: Border?
    border_radius: f32?

    # Sombra
    shadow: Shadow?
    shadow_inner: Shadow?

    # Tamaño
    width: Length?
    height: Length?
    min_width: Length?
    max_width: Length?
    min_height: Length?
    max_height: Length?

    # Layout
    display: Display?
    flex_direction: FlexDirection?
    flex_wrap: bool?
    flex_grow: f32?
    flex_shrink: f32?
    align_items: Alignment?
    align_self: Alignment?
    justify_content: Alignment?
    gap: f32?
    grid_columns: i32?
    grid_rows: i32?
    grid_column_span: i32?
    grid_row_span: i32?

    # Posición
    position: Position?
    top: Length?
    left: Length?
    right: Length?
    bottom: Length?
    z_index: i32?

    # Efectos
    opacity: f32?
    cursor: Cursor?         # Pointer, Default, Text, Wait, Crosshair, NotAllowed
    pointer_events: bool?
    overflow: Overflow?

    # Transformación
    transform: Transform?
    transform_origin_x: f32?
    transform_origin_y: f32?
    transition: Transition?
    animation: Animation?

    # Clip
    clip: bool?               # clip contenido al borde
    clip_path: str?            # SVG clip path
```

---

## 3. Declaración de Estilos

### 3.1 Styles

```kyle
# Estilo base
style<button> Primary:
    background = Color("#0066FF")
    color = Color("#FFFFFF")
    font_size = 14
    font_weight = FontWeight.Bold
    padding = Spacing.all(12)
    border_radius = 8
    cursor = Cursor.Pointer

# Estilo que hereda de otro
style<button> PrimaryHover: Primary:
    background = Color("#0052CC")

# Estilo variante
style<button> Secondary:
    background = Color("transparent")
    color = Color("#0066FF")
    border = Border(2, Color("#0066FF"), BorderStyle.Solid)
    padding = Spacing.all(12)
    border_radius = 8
```

### 3.2 Layouts

```kyle
layout<column> Center:
    align_items = Alignment.Center
    justify_content = Alignment.Center
    gap = 16

layout<row> SpaceBetween:
    justify_content = Alignment.SpaceBetween
    align_items = Alignment.Center
```

### 3.3 Templates (combinan style + layout + animation)

```kyle
tpl<button> Primary:
    style = Primary
    animation = ripple_animation
    cursor = Cursor.Pointer
    ripple = true

tpl<card> Elevated:
    style = elevated_card_style
    animation = hover_elevation
    shadow = Shadow(0, 4, 8, 0, Color.black().with_alpha(0.15))
```

### 3.4 Themes

```kyle
theme LightTheme:
    # Colores
    primary = Color("#0066FF")
    on_primary = Color("#FFFFFF")
    primary_container = Color("#D6E4FF")
    secondary = Color("#FF6600")
    on_secondary = Color("#FFFFFF")
    background = Color("#FFFFFF")
    on_background = Color("#1A1A1A")
    surface = Color("#F5F5F5")
    on_surface = Color("#1A1A1A")
    error = Color("#DC3545")
    on_error = Color("#FFFFFF")
    outline = Color("#CCCCCC")

    # Tipografía
    font_display = TextStyle(
        font_family: "Inter",
        font_size: 32,
        font_weight: FontWeight.Bold,
    )
    font_headline = TextStyle(
        font_family: "Inter",
        font_size: 24,
        font_weight: FontWeight.SemiBold,
    )
    font_title = TextStyle(
        font_family: "Inter",
        font_size: 18,
        font_weight: FontWeight.Medium,
    )
    font_body = TextStyle(
        font_family: "Inter",
        font_size: 14,
        font_weight: FontWeight.Normal,
    )
    font_caption = TextStyle(
        font_family: "Inter",
        font_size: 12,
        font_weight: FontWeight.Normal,
    )

    # Spacing
    spacing_xs = 4
    spacing_sm = 8
    spacing_md = 16
    spacing_lg = 24
    spacing_xl = 32
    spacing_xxl = 48

    # Border radius
    radius_sm = 4
    radius_md = 8
    radius_lg = 16
    radius_full = 9999

    # Breakpoints
    breakpoint_sm = 640
    breakpoint_md = 1024
    breakpoint_lg = 1280

theme DarkTheme: LightTheme:
    primary = Color("#4D8EFF")
    background = Color("#121212")
    on_background = Color("#E0E0E0")
    surface = Color("#1E1E1E")
    on_surface = Color("#E0E0E0")
```

---

## 4. Uso de Estilos en Componentes

### 4.1 Aplicar estilo

```kyx
<button tpl=Primary text="Ingresar" />
<button style=Secondary text="Cancelar" />
<card tpl=Elevated>
    <text value="Contenido" />
</card>
```

### 4.2 Estilos inline (casos específicos)

```kyx
<text
    style=Style(color: Color("#FF0000"), font_size: 16)
    value="Error"
/>
```

### 4.3 Combinar estilos

```kyx
<button
    style=Style.combine(Primary, Style(margin: Spacing.all(8)))
    text="Combinado"
/>
```

---

## 5. Compilación por Target

### 5.1 Web target

Los estilos Kyle se compilan a inline styles + CSS custom properties:

```kyle
style<button> Primary:
    background = Color("#0066FF")
    color = Color("#FFFFFF")
    border_radius = 8
```

→ JavaScript generado:
```javascript
// ui-runtime.js (generado automáticamente)
const styles = {
  Primary: {
    background: '#0066FF',
    color: '#FFFFFF',
    borderRadius: '8px',
    fontFamily: 'Inter, sans-serif',
    fontSize: '14px',
    fontWeight: 'bold',
    padding: '12px 24px',
    cursor: 'pointer',
  }
};

function applyStyle(element, styleName) {
  const s = styles[styleName];
  for (const [prop, val] of Object.entries(s)) {
    element.style[prop] = val;
  }
}
```

### 5.2 Desktop target (Skia)

```kyle
style<button> Primary:
    background = Color("#0066FF")
    border_radius = 8
    padding = Spacing(12, 24)
```

→ Código Kyle generado:
```kyle
# Generado automáticamente por el compilador de estilos
fn render_primary_button(x: f32, y: f32, w: f32, h: f32):
    skia_draw_rounded_rect(x, y, w, h, 8.0, Color("#0066FF"))
    # padding: el texto se dibuja en (x + 12, y + 24)
```

### 5.3 Mobile target

Los estilos se traducen a APIs nativas de cada plataforma:

**Android:**
```java
// Generado automáticamente
Button btn = new Button(context);
btn.setBackgroundColor(Color.parse("#0066FF"));
btn.setTextColor(Color.WHITE);
btn.setPadding(12, 24, 12, 24);
```

**iOS:**
```swift
// Generado automáticamente
let btn = UIButton()
btn.backgroundColor = UIColor(hex: "#0066FF")
btn.setTitleColor(.white, for: .normal)
btn.contentEdgeInsets = UIEdgeInsets(12, 24, 12, 24)
```

---

## 6. Responsive

### 6.1 Media queries en Kyle

```kyle
style<view> Card:
    width = Length.Percent(100)
    padding = Spacing.all(8)

    @media(min_width: 640):
        width = Length.Px(400)
        padding = Spacing.all(16)

    @media(min_width: 1024):
        max_width = Length.Px(800)
        margin = Spacing(left: Length.Auto, right: Length.Auto)
```

### 6.2 Compilación responsive

Web target → `@media` queries en CSS:
```javascript
// Generado automáticamente
const styles = {
  Card: {
    base: { width: '100%', padding: '8px' },
    '@media (min-width: 640px)': { width: '400px', padding: '16px' },
    '@media (min-width: 1024px)': { maxWidth: '800px', margin: '0 auto' },
  }
};
```

Desktop target → el layout engine maneja breakpoints en tiempo de render:
```kyle
fn render_card(viewport_w: f32):
    style = if viewport_w >= 1024:
        card_desktop_style
    elif viewport_w >= 640:
        card_tablet_style
    else:
        card_mobile_style
```

---

## 7. Modo oscuro

```kyx
<view>
    @(theme = current_theme())
    <text value=if theme == DarkTheme: "Modo oscuro" else: "Modo claro" />
    <button
        text="Toggle theme"
        click=@toggle_theme
    />
</view>
```

El theme se compila a variables que cambian dinámicamente:

Web:
```javascript
document.documentElement.style.setProperty('--primary', '#4D8EFF');
```

Desktop:
```kyle
theme_manager.set_active(DarkTheme)
```

---

## 8. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Animaciones
