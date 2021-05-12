
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