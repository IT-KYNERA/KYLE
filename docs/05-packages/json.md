# json — JSON Parsing and Generation

**Status:** Available in runtime, planned as package

## API

```ky
from json import parse, stringify

data = parse("{\"name\": \"Ana\", \"age\": 30}")
println(data["name"])   # "Ana"

text = stringify(data)
println(text)           # '{"name":"Ana","age":30}'
```
