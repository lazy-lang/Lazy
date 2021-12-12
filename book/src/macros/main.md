
# Metaprogramming

Lazy allows **metaprogramming** in two ways:

- Function macros (very similar to Rust's `macro_rules!`)
- "Meta" statements

## Function Macros

Macros are defined with the `macro` keyword, they act exactly like a function, except that any call to the macro will be **expanded** to the code inside the macro. Here's an example:

```
macro vec {
    ($len: num) => {
        Vec::from_len($len);
    },
    (+item: expr;) => {{
        let v = Vec::new();
        +;(
            v.push($item);
        )
        v;
    }}
}

main {
    let my_vector = #vec(10);
    let my_full_vec = #vec(1, 2, 3);
}
```

turns to:

```
main {
    let my_vector = Vec::from_len(10);
    let my_full_vec = {
        let v = Vec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v
    }
}
```

Every macro has a set of **arms**, which can contain different arguments or set of characters, and expand to entirely different code.

This allows for easily creating DSLs (Domain Specific Languages):

```
// Markdown format
macro mkd_fmt {
    ##$name: expr## => { // Heading
        `# ${$name}`;
    }
    __$name: expr__ => { // Underline
        `__${$name}__`;
    }
}

main {
    let title = #mkd_fmt ##"Article Name"##;
    let important_info #mkd_fmt __"This is some important info"__;
}
```