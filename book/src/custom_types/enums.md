
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
    let str_token = TokenTypes:String("some token type");
    str_token == TokenTypes:String; // returns true

    // str_token gets automatically unwrapped
    if str_token == TokenTypes:String print(str_token) // prints "some token type"
    else if str_token == TokenTypes:Integer print(str_token + 5)

    let num_token = TokenTypes:Number(Number:Int(3000));
}
```