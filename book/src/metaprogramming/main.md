
# Metaprogramming

Lazy allows you to create **function macros** which allow you to write cleaner and less-repetitive code. They're very similar to rust macros:

```
macro my_macro($a: ident, $b: expr) => {
    $a[$b];
}

main {

}
```