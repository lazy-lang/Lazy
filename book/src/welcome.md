# Welcome

This book acts as the **specification** for the **Lazy** programming language! Every detail about the language (grammar, data types, as well as internals such as memory management and optimizations) is provided, as well as examples. 

Lazy is a statically typed, multi-paradigm general purpose programming language with a strucutral typing system, which compiles to machine code. It's a language heavily inspired by Rust, Typescript and purely functional languages like F# and Haskell.

## A taste

```

fn add(num1: i32, num2: i32) -> i32 {
    num1 + num2
}

main {
    add(1, 5) 
    |> print
}
```


