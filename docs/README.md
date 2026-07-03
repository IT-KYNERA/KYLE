# Kyle Language — Official Documentation

Welcome to the official documentation for the Kyle programming language.

## Documentation structure

| Section | Description |
|---------|-------------|
| [01-overview](01-overview/README.md) | Vision, philosophy, principles, ecosystem architecture |
| [02-guide](02-guide/README.md) | Tutorial: learn Kyle step by step |
| [03-language-reference](03-language-reference/README.md) | Formal language specification |
| [04-platform](04-platform/README.md) | Compiler, standard library, tools |
| [05-packages](05-packages/README.md) | Official package specifications |
| [06-reference](06-reference/README.md) | Quick reference: keywords, flags, CLI |
| [07-engineering](07-engineering/README.md) | Compiler architecture and implementation |
| [08-design](08-design/README.md) | Design decisions, ADRs, RFCs |
| [09-project](09-project/README.md) | Roadmap, versioning, license |
| [10-history](10-history/README.md) | Changelog, migration guides |

## Conventions

- `.ky` = Kyle source file
- `this` = instance reference
- `T?` = optional type (`Option<T>`)
- `T!` = fallible type (`Result<T, Error>`)
- `&T` = mutable type
- `^T` = move type (ownership transfer)
