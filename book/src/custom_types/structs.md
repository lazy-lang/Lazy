
# Structs

Structs contain key-value pairs called `fields`. All keys and the types of the values must be known at compile-time.

```
struct Something<GenericParam> {
    val: GenericParam,

    set: fn(val: GenericParam) -> bool {
        self.val = val;
        true;
    }

}
```

## Field modifiers

### Static fields

Sometimes you want a field to be attached to the struct itself and not an instance of the struct.

```
struct Person {
    name: str,

    static new: fn(name: str) -> Person {
        new Person { name };
    }

}

let me = Person.new("Google");
```

### Hidden/Private fields

Fields that can only be accessed in the execution context of the functions inside the sctruct.

```
struct Person {
    name: str,
    private id: i32

    static new: fn(name: str) -> Person {
        new Person { name, id: rng() };
    }

}

let me = Person.new("Google");
me.id; // Error!
```

### Immutable fields

Fields that cannot be mutated.

```
struct Person {
    const name: str,
    private id: i32

    static new: fn(name: str) -> Person {
        new Person { name, id: rng() };
    }

}

let me = Person.new("Google");
me.name = "Pedro"; // Error!
```

### Optional fields

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
    let my_smth = new Something<str>{val: "Hello"};
    my_smth.set(val: "World");
    my_smth.val.length; // returns 5
}
```

## Accessing optional fields

Optional fields can be accessed by the dot notation, but you must put a question mark (`?`) before the dot. If the optional struct is empty, then the expression returns `none`.

```
main {
    let my_struct = new StructWithOptionalField{}; // the optional field is `none`
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