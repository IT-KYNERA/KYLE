# xml — XML

> Módulo de parseo y generación XML.
> Import: `from xml import xml`

## xml: parseo y generación

```ky
from xml import xml

doc: xml = xml.parse('<root><item id="1">Hello</item></root>')
items: {xml} = doc.find_all("item")
first: xml = items[0]
texto: str = first.text()
id_val: str = first.attr("id")
```

### Funciones

| Función | Firma | Descripción |
|---------|-------|-------------|
| `xml.parse(str)` | `fn(s: str) xml` | Parsear string → documento |
| `xml.element(name)` | `fn(name: str) xml` | Crear elemento |

### Métodos (nodo)

| Método | Firma | Descripción |
|--------|-------|-------------|
| `n.find_all(tag)` | `fn(self, tag: str) {xml}` | Buscar hijos por tag |
| `n.find_first(tag)` | `fn(self, tag: str) xml?` | Primer hijo que matchea |
| `n.text()` | `fn(self) str` | Texto del nodo |
| `n.attr(name)` | `fn(self, name: str) str` | Valor de atributo |
| `n.set_attr(name, val)` | `fn(self, name: str, val: str)` | Asignar atributo |
| `n.add_child(child)` | `fn(self, child: xml)` | Agregar hijo |
| `n.set_text(text)` | `fn(self, text: str)` | Asignar texto |
| `n.to_str()` | `fn(self) str` | Serializar a string |
