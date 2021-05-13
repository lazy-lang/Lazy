
# Primitives

Much like the rest programming languages, Lazy provides a bunch of primitive data types.

## Scalar types

- Signed integers: `i8`, `i16`, `i32`
- Unsigned integers: `u8`, `u16`, `u32`
- Floating points: `f32`
- Characters: `char`
- Strings: `str`
- Booleans: `true` or `false`
- The `none` data type

## Compound types

### Tuples

Tuples can hold values of different types:

```
const items<|str, i32, bool|> = |"Hello", 64, true|;

items.1; // 64
```

