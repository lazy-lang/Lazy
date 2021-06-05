# Partials

`Lazy` does not have core OOP design patters like inheritance, but it has `Partials`, which are very similar to rust's `Traits`, but a little more dynamic.

## Partial struct

```
struct Animal {
    name: str,
    age: i8,

    make_noise: fn(noise?: str) print(noise || "*quiet*")
}

struct Human {
    name: str,
    age: i8,
    job: str

    make_noise: fn() print("Hello World")
}

type WithName = { name: str };

main {
    // The function will only have access to the name field
    let get_name = fn(thing: WithName) -> str {
        thing.name;
    }

    let me = new Human{name: "Google", job: "Programmer", age: 19};
    let some_animal = new Animal{name: "Perry", age: 2};

    get_name(me); // Google
    get_name(some_animal); // Perry
}
```

## Combining types

Partials can be combined to create more complex partials:

```
type Stringable = {
    to_string: () -> str
}

type Numberable = {
    to_int: () -> i32
}

static num_and_str = fn(val: Stringable + Numerable) -> [str, i32] {
    [val.to_string(), val.to_int()];
}
```


## Partial function

```
// Requires the type to have a "make_noise" method - we don't care about params or return value in this case
type MakesNoise = { make_noise: () };

main {
    let me = new Human{name: "Google", job: "Programmer", age: 19};
    let some_animal = new Animal{name: "Perry", age: 2};

    let stuff_that_makes_noise = new Vec<MakesNoise>{};
    stuff_that_makes_noise.push(me);
    stuff_that_makes_noise.push(some_animal);

    stuff_that_makes_noise[0]?.make_noise(); // "Hello World"
    stuff_that_makes_noise[1]?.make_noise(); // *quiet*
}
```