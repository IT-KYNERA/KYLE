# Classes

## final class

Lightweight struct with fields. No inheritance.

```ky
final class Vec2:
    x: i32
    y: i32
```

### Construction

The default constructor assigns fields by name:

```ky
v = Vec2 { x: 10, y: 20 }
```

### Explicit constructor

```ky
final class Config:
    name: str
    port: i32

    fn new(name: str, port: i32 = 8080) Config:
        Config { name, port }
```

## class

Full class with single inheritance.

```ky
class Animal:
    name: str

    fn speak(this):
        println("...")
```

### Inheritance

```ky
class Dog :: Animal:
    fn speak(this):
        println("woof")
```

### Abstract class

Cannot be instantiated. Serves as a base for subclasses.

```ky
abstract class Shape:
    fn area(this) f64
```

## Mutable fields

```ky
final class Config:
    name: str
    port: &i32          # mutable field

config = Config { name: "server", port: 8080 }
config.port = 9090      # allowed
```

## Properties

```ky
final class Person:
    _name: str

    prop name:
        get:
            this._name
        set:
            this._name = value
```
