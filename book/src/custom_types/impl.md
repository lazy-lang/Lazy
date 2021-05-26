
# Impl

The **impl** keyword is used to combine a struct or an enum with a **partial** type, so the struct/enum fits the partial. Generally, this keyword doesn't have to be used with structs in order to make a struct compatible with a type, but it's nice syntactic sugar. 

```
struct Human {
    age: i8
}

struct Animal {
    age: i8,
    species: AnimalTypes
}

type Speak = { speak: (...args?: str) -> str } // A partial which requires the structure to have a "speak" method which returns a `str`

impl Speak for Human {
    speak: fn(...args?: str) -> str {
        if args.0 args.0;
    }
}

impl Speak for Animal {
    speak: fn(...args?: str) -> str {
        match self.species {
            AnimalTypes::Cat => "meow",
            AnimalTypes::Dog => "woof",
            AnimalTypes::Mouse => "*squick*"
        }
    }
}

```

## Type guards

This feature can also be used to create type guards:

```
type TotalStrLen {
    total_len: () -> i32
}

impl TotalStrLen for Vec<str> {
    total_len: fn() -> i32 {
        let len = 0;
        for string in self {
            len += string.length;
        }
        len;
    }
}

main {
    let vec_of_strs = Vec::from<str>("a", "b", "cde");
    vec_of_strs.total_len(); // 5
    let vec_of_nums = Vec::from<i32>(1, 2, 3, 4, 5);
    vec_of_nums.total_len(); // Error!
}
```

## Require `impl`

Sometimes, the creator of a partial may want it to be explicitly implemented via the `impl` keyword. Place a `!` after the partial type in order to make sure that the provided struct fits the partial:

```
type Add<Other, Output> = {
    add: (other: Other) -> Output
}

static add<Other, Output>(a: Add<Other, Output>!, b: Other) -> Output {
    a.add(b);
}

struct Token {
    value: char,

    static create: fn(value: char) -> Token {
        new Token { value };
    }

    add: fn(other: Token) -> Token {
       new Token { value: self.value + other.value };
    }
}

main {
    let tok1 = Token::create('a');
    let tok2 = Token::create('b');
    add<Token, Token>(tok1, tok2); // Invalid, because "Token" doesn't explicitly fit `Add`!
}
```

Correct implementation:

```
struct Token {
    value: char
}

impl Add<Token, Token> for Token {
    add: fn(other: Token) -> Token {
        new Token { value: self.value + other.value };
    }
}

main {
    add<Token, Token>(Token::create('a'), Token::create('b')); // "ab"
}
```