
# Custom Types

There are two ways to create custom types in **Lazy**:

- `enum` - For creating enumerations with different variants, which can hold values of different types
- `struct` - Structs are very similar to C structs, except they can contain functions

## Type aliasing

Type aliasing can be done by using the `type` keyword:

```
type i = i8;

main {
    let a<i> = 4;
}
```

## None-able types

A type which is prefixed with `?` is `none`-able. The value it may have may be `none` or the type.

```
static div = fn(a: i32, b: i32) -> i32? {
    if b == 0 none
    else a / b
}
```