# VS Code Extension — Kyle Language Support

**File:** `vscode-ky/`

---

## 1. Architecture

```
vscode-ky/
├── package.json                  # Manifest: languages, grammars, commands, debugger, tasks
├── language-configuration.json   # Comments, brackets, auto-closing, indentation, folding
├── syntaxes/
│   ├── ky.tmLanguage.json        # TextMate grammar for .ky (Kyle language)
│   └── kyx.tmLanguage.json       # TextMate grammar for .kyx (Kyle UI)
├── snippets/
│   ├── ky.json                   # 53 snippets for .ky files
│   └── kyx.json                  # 48 snippets for .kyx files
├── src/
│   ├── extension.ts              # LSP client, commands, diagnostics, binary discovery
│   ├── tasks.ts                  # VS Code Task provider
│   ├── debugger.ts               # Debug Adapter Protocol implementation
│   └── testUI.ts                 # Testing UI integration
├── themes/
│   └── kl-color-theme.json       # "Kyle Pastel" dark theme
└── icons/
    ├── ky.png
    └── ky_128.png
```

---

## 2. Languages

| Language ID | File Extensions | Grammar |
|-------------|----------------|---------|
| `kl` | `.ky`, `ky.toml` | `syntaxes/ky.tmLanguage.json` |
| `kyx` | `.kyx` | `syntaxes/kyx.tmLanguage.json` |

---

## 3. .kyx Grammar (`kyx.tmLanguage.json`)

### 3.1 Top-level patterns (in order)

| Pattern | Description |
|---------|-------------|
| `#comments` | `##` doc comments, `#` line comments |
| `#import_decl` | `from views.home import home` |
| `#link_directive` | `@link "SDL2"` |
| `#style_decl` | `style<button> Primary:`, `layout<view>`, `animation<view>` |
| `#theme_decl` | `theme LightTheme:` |
| `#kyle_block` | `@(...)` multi-line code block |
| `#at_directive` | `@if(...):`, `@for(...):`, `@match(...):` |
| `#xml_element` | `<tagname>...</tagname>` (recursive, includes closing tag coloring) |
| `#xml_self_closing` | `<tagname .../>` |
| `#xml_slot` | `<slot />` |
| `#kyle_inline` | `@expr` inline expressions |

### 3.2 XML tag names — coloring

- **Opening tag**: `<view>`, `<vstack>`, `<app>` → `entity.name.tag.kyx`
- **Closing tag**: `</view>`, `</vstack>` → `entity.name.tag.kyx` (via `#closing_tag` pattern)
- **Self-closing**: `<button />` → `entity.name.tag.kyx`
- **Attributes**: `style=`, `click=`, `field=` → `entity.other.attribute-name.kyx`
- **Attribute values**: `"Primary"`, `"image/*"` → `string.quoted.double.kyx`
- **Expression attrs**: `@handler`, `@count.to_str()` → `keyword.operator.kyx` prefix

### 3.3 Import syntax

```
from views.home import home
└────┘ └──────────┘ └──────┘ └────┘
  keyword   module     keyword  name
```

- `from`, `import` → `keyword.control.import.kyx`
- Module path → `entity.name.module.kyx`
- Imported name → `entity.name.function.kyx`

### 3.4 Code blocks `@(...)` and `@expr`

Inside `@(...)`, the following are highlighted:
- **Types**: `i32`, `str`, `bool`, `color`, `spacing`, `file_data`, `bytes`, etc.
- **Keywords**: `fn`, `if`, `for`, `while`, `return`, `and`, `or`, `not`, `is`
- **Builtins**: `print`, `navigate`, `set_title`, `set_meta`, `color`, `spacing`, etc.
- **Operators**: `+=`, `==`, `!=`, `->`, `::`, `?.`
- **Numbers**: `42`, `0xFF`, `0b1010`
- **Strings**: `"hello {name}"` with interpolation
- **Variables**: `counter`, `name: ^i32`

### 3.5 Style/animation declarations

```
style<text> Title:       layout<view> Center:       animation<view> FadeIn:
└─────┘ └────┘ └─────┘   └──────┘ └────┘ └──────┘   └────────┘ └────┘ └──────┘
keyword  comp   name      keyword   comp   name      keyword    comp   name
```

---

## 4. .ky Grammar (`ky.tmLanguage.json`)

### 4.1 Key features

| Feature | Pattern |
|---------|---------|
| Comments | `##`, `#` |
| Strings | `"..."` with `{interpolation}` |
| `@link` | `@link "library"` |
| Characters | `'a'`, `'\n'` |
| Numbers | `0xFF`, `0b1010`, `42`, `3.14` |
| Attributes | `#[test]`, `#[deprecated]` |
| Class def | `final class Name:`, `abstract class Name:` |
| Enum def | `enum Name:` |
| Contract def | `contract Name:` |
| Type alias | `type Name = ...` |
| Extern fn | `extern fn name(...) type` |
| Function def | `fn name(...) type:`, `static fn`, `async fn` |
| Imports | `from http.server import Router` |
| Constructor | `TypeName(...)` |
| Keywords | control flow, declarations, operators |
| Types | `i32`, `str`, `bool`, `^T`, `&T`, `T?`, `T!` |
| Builtins | `print`, `len`, `map`, `filter`, `serialize`, etc. |

---

## 5. Snippets

### 5.1 .kyx snippets (48 total)

| Prefix | Component | Description |
|--------|-----------|-------------|
| `app` | App entry | `from views.home import home` + router |
| `view` | View | `<view>$0</view>` |
| `vstack` | VStack | Vertical flex layout |
| `hstack` | HStack | Horizontal flex layout |
| `zstack` | ZStack | Overlay layout |
| `text` | Text | `<text value="..." />` |
| `button` | Button | With `style=Primary` + `click=@handler` |
| `link` | Link | `<link to="/">text</link>` |
| `text_field` | TextField | With `bind=` or `field=` |
| `password_field` | PasswordField | Password input |
| `text_area` | TextArea | Multi-line input |
| `checkbox` | Checkbox | Toggle |
| `radio` | Radio | Option |
| `switch` | Switch | Toggle switch |
| `slider` | Slider | Range slider |
| `image` | Image | `<image src="..." />` |
| `spacer` | Spacer | `<spacer />` |
| `divider` | Divider | `<divider />` |
| `scroll` | Scroll | Scrollable container |
| `modal` | Modal | Dialog overlay |
| `sheet` | Sheet | Bottom sheet |
| `alert` | Alert | Alert dialog |
| `form` | Form | With `model=` + `submit=` |
| `field` | Form field | `<text_field field="name">` |
| `file_picker` | FilePicker | Native file picker |
| `file_picker_cb` | FilePicker cb | With inline callback |
| `router` | Router | Route container |
| `route` | Route | `<route path="/" component=home layout=main>` |
| `layout` | Layout | Persistent layout with `<slot />` |
| `slot` | Slot | `<slot />` |
| `navbar` | Navbar | Navigation bar |
| `sidebar` | Sidebar | Side panel |
| `import` | Import | `from views.name import name` |
| `style` | Style | `style<text> Name: prop = value` |
| `code` | Code block | `@(...)` |
| `@if` | If | `@if(cond): ... @else:` |
| `@for` | For | `@for(item in list):` |
| `@match` | Match | `@match(expr): Pattern:` |
| `click` | Click attr | `click=@handler` |
| `bind` | Bind attr | `bind=@variable` |
| `model` | Model attr | `model=@form` |
| `target` | Target config | `target(Target.web): port = 8080` |
| `fn` | Function | `fn name(args):` |
| `on_click` | Inline event | `click=@fn ():` |

### 5.2 .ky snippets (53 total)

From `snippets/ky.json` — includes function, class, enum, contract, import, async, debugging, and type snippets.

---

## 6. Semantic Highlighting (LSP)

The LSP server (`ky lsp`) provides semantic tokens for:
- Variables (mutable, immutable)
- Functions (declarations, calls)
- Types (built-in, user-defined)
- Keywords
- Parameters

Configured via `ky.semanticHighlighting` setting (default: `true`).

---

## 7. Commands

| Command | Title | Action |
|---------|-------|--------|
| `ky.run` | KL: Run current file | Terminal: `ky run <file>` |
| `ky.build` | KL: Build current file | Terminal: `ky build <file>` |
| `ky.check` | KL: Type-check file | Terminal: `ky check <file>` |
| `ky.test` | KL: Run tests | Terminal: `ky test <file>` |
| `ky.runTest` | KL: Run specific test | Wraps test + runs |

---

## 8. Tasks

`KyleTaskProvider` discovers `*.ky` files and creates:
- `kl: run <file>` → `ky run <file>`
- `kl: build <file>` → `ky build <file>`
- `kl: check <file>` → `ky check <file>`

---

## 9. Debugger

Uses Debug Adapter Protocol. Configured in `launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "KL: Launch",
      "type": "kl",
      "request": "launch",
      "program": "${workspaceFolder}/src/main.ky",
      "klcPath": "ky"
    }
  ]
}
```

---

## 10. Testing UI

- Discovers `#[test]` annotated functions via regex
- Creates VS Code Test Controller items
- Run profile: compiles + executes binary
- Debug profile: same runner with debug output
- Watches filesystem for `.ky` changes

---

## 11. Theme: "Kyle Pastel"

Dark theme with:
- Background: `#1a1a2e` (dark navy)
- Accent: `#e94560` (coral red)
- Strings: `#7bed9f` (green)
- Functions: `#70a1ff` (blue)
- Types: `#a29bfe` (purple)
- Numbers/constants: `#f9ca24` (yellow)
- Operators: `#f8a5c2` (pink)
- Comments: `#6a6a8a` (muted purple, italic)

---

## 12. Building the Extension

```bash
cd vscode-ky
npm install
npx tsc -p tsconfig.json           # Compile TypeScript
npx @vscode/vsce package            # Create .vsix
code --install-extension ky-*.vsix  # Install locally
```
