
# Functions

In Lazy, functions are first-class citizens, this means they can be passed as function arguments, stored easily in data structures, and be saved in variables. 

```
main {
    const say_alphabet_lowercase = fn () print(...'a'..='z');
    say_alphabet_lowercase(); 
}
```

A function may have up to 255 arguments!

## Function return values

A return value type can be specified to any function by adding and arrow and the type after the arguments part:

```
const is_ok = fn() -> bool {
    true;
}
```

As you notice, there's no `return` statement in `Lazy`. The last expression in the function body always gets returned. Everything in `Lazy` is an expression, except `type`, `struct` and `enum` declarations, so it's all good. 

If a return type is not provided, the function will always return `none`. 

## Execution context

All functions in `Lazy` have an **execution context** which can be referenced via `self` in the function body. The execution context of functions is always `none`, unless the function is defined inside a structure:

```
struct Car {
    model: str,
    beep_noise: str,

    beep: fn() {
        print(self.beep_noise); 
    }
}
```

