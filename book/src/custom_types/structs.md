
# Structs

Structs contain key-value pairs called `fields`. All keys and the types of the values must be known at compile-time.

```
struct Something<GenericParam> {
    val: GenericParam,

    // Empty constructor - fallbacks to the default
    Something: fn() {},

    set: fn(val: GenericParam) -> bool {
        self.val = val;
        true;
    }

}
```

## Constructors

The method with the same name as the struct acts as the **constructor**.

## Optional fields

Fields which have a question mark (`?`) after the key are **optional**.

```
struct StructWithOptionalField {
    // Can either be "none", or "str"
    something?: str
}
```

## Accessing fields

Fields are accessed using the dot notation (`.`).

```
main {
    let my_smth = Something<str>{val: "Hello"};
    my_smth.set(val: "World");
    my_smth.val.length; // returns 5
}
```

## Accessing optional fields

Optional fields can be accessed by the dot notation, but you must put a question mark (`?`) before the dot. If the optional struct is empty, then the expression returns `none`.

```
main {
    let my_struct = StructWithOptionalField{}; // the optional field is `none`
    my_fn(my_struct.something?); // The function doesn't get executed because my_struct cannot be unwrapped
    my_struct.something = "Hello";
    my_fn(my_struct.something?); // Runs fine!
}
```

Keep in mind that you **cannot** use optional fields before you make sure they are not `none`. 

```
if my_struct != none {
    // my_struct is guaranteed to not be none here, no need for question marks!
    print(my_struct.something);
}
```