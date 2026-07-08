# xml — XML

> Módulo de parseo y generación XML.
> Import: `from xml import xml`

## xml: parseo y generación

```ky
from xml import xml

# Parsear XML
doc = xml.parse('<root><item id="1">Hello</item></root>')
items = doc.find_all("item")
first = items[0]
println(first.text())      # "Hello"
println(first.attr("id"))  # "1"

# Generar XML
doc = xml.element("root")
item = doc.add_child("item")
item.set_attr("id", "1")
item.set_text("Hello")
str = doc.to_str()
```

### Funciones

| Función | Descripción |
|---------|-------------|
| `xml.parse(str)` | Parsear string → documento |
| `xml.element(name)` | Crear elemento |

### Métodos (nodo)

| Método | Descripción |
|--------|-------------|
| `n.find_all(tag)` | Buscar hijos por tag |
| `n.find_first(tag)` | Primer hijo que matchea |
| `n.text()` | Texto del nodo |
| `n.attr(name)` | Valor de atributo |
| `n.set_attr(name, val)` | Asignar atributo |
| `n.add_child(child)` | Agregar hijo |
| `n.set_text(text)` | Asignar texto |
| `n.to_str()` | Serializar a string |
