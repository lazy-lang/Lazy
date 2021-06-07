
# Logic

## if/else

In Lazy, if/else conditionals are expressions. All branches of the condition **must** return the same type, or none. Any expression can be used for the condition and the body - even other if expressions.

```
const a = 10;

if a > 10 print("a is more than 10")
else if a > 5 print("a is more than 5")
else print("a is less than 5")
```

```

enum Number {
    Int: i32,
    Float: f32
}

let enum_field = Number::Int(15);

// if "enum_field" is of type `Number::Float`, return the unwrapped float inside it, otherwise return 0.
let my_num = if enum_field == Number:Float enum_field else 0;

print(my_num == 0) // true
```

## match

The `match` expression is exactly the same as Rust's `match`.

```
let my_num = Number:Float(3.14);

match my_num {
    Number::Float => print("Found float ", my_num),
    Number::Int => print("Found integer ", my_num),

    // Specific cases
    Number::Float(3.14) => print("Found pi!"),

    // Guards
    Number::Int if my_num > 10 => print("Found integer bigger than 10),

    // Execute the same body for different expressions
    Number::Int(3) | Number::Float(3.1) => print("Number is ", my_num),

    // Acts as an "else" - only executed when nothing above gets matched
    _ => print("Weird...")
}
```

The contents of the `match` expression are called `arms` - each arm has a condition (which can be followed by a guard) and a body. 

A condition can be:

- Enum variants (`Enum::variant`)
- Literals (`"hello"`, `'c'`, `3`, `45.3`, `[1, 2, 3]`, `true`, `false`, `none`)
- Range iterators (`0..10`, `5..=1000`, `'a'..='z'`)
- A list of expressions separated by `|` (`1 | 5 | 7`) - if not all expressions return the same type, the value doesn't get unwrapped, but the body still gets executed.

Guards can be any expression.

A body can be any expression. 