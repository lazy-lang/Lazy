
# Enums

Enums are types which may be one or a few different variants.

```
enum Number {
    Int: i32,
    Float: f32
}

enum TokenTypes {
    String: str,
    Integer: i32,
    Number: Number,
    Character: char,
    Invalid
}

main {
    let str_token = TokenTypes::String("some token type");
    str_token == TokenTypes::String; // returns true

    // str_token gets automatically unwrapped
    if str_token == TokenTypes::String print(str_token) // prints "some token type"
    else if str_token == TokenTypes::Integer print(str_token + 5)

    let num_token = TokenTypess::Number(Numbers::Int(3000));
}
```

## "Unwrapping" enums

By default all items are "wrapped" - their values and unknown and can be anything.

```
let char_token = TokenTypes::Char('a'); 
// Even if it's clear that char_token is of type `char`, Lazy doesn't allow you to use it. 
```

To use the value inside the variant you'll have to "unwrap" it.

### if unwrapping

```
if char_token == TokenTypes::Char {
    // You can use `char_token` as a character now.
    char_token.to_string(); // "a"
}
// char_token is wrapped here.

if char_token == TokenTypes::Char('a') {
    // Also works!
}
```

### match unwrapping

```
match char_token {
    TokenTypes::Char => {
        // char_token is unwrapped here
    },
    _ => {
        // char_token is wrapped, this portion only executes when nothing else got matched
    }
}
```

## Attaching methods and properties to enums

Attaching methods and properties to enums is done via the **impl** keyword. 

```
enum Option<T> {
    Some: T,
    None
}

type Unwrap<T> = {
    unwrap: () -> T,
    is_some: () -> bool,
    unwrap_or: (v: T) -> T
}

impl<T> Unwrap<T> for Option<T> {
    unwrap: fn() -> T {
        match self {
            Some => self,
            None => error("Tried to unwrap an empty option!")
        }
    }

    is_some: fn() -> bool {
        self == Option::Some
    }

    unwrap_or: fn(v: T) -> T {
        match self {
            Some => self,
            None => v
        }
    }
}

main {
    let maybe<Option<i32>> = Option::None;
    maybe.unwrap_or(0); // 0
    maybe = Option::Some(32);
    maybe.is_some(); // true
    maybe.unwrap(); // 32
}
```