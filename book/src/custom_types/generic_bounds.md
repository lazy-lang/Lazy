
# Generic bounds

In `struct`s, `enum`s and `type`s, generic parameters can be **bounded**. That means that only a type which satisfies the specified partial can be passed as a generic parameter.

```
struct Error<
    T: { to_string: () -> str }
> {
    content: T,
    type: i32,
    format: fn() -> str {
        `Error ${self.type}: ${self.content}`;
    }

}

struct CustomErrorContent {
    vars: Vec<str>,
    to_string: fn() -> str {
        vars.join(", ");
    }
}

main {
    const my_custom_error_content = new CustomErrorContent { vars: Vec::from("Hello", " ", "World!") };
    const error = new Error<CustomErrorContent> { content: my_custom_error_content, type: 1 };
    print(error.format()); // "Error 1: Hello World"
}
```
