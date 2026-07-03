# ky-postgres — PostgreSQL Driver

**Status:** Planned

## API

```ky
from ky-postgres import Connection

conn = Connection.open("host=localhost dbname=test user=postgres")
rows = conn.query("SELECT * FROM users")
for row in rows:
    println(row)
```
