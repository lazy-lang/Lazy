# Partials

`Lazy` does not have core OOP design patters like inheritance, or `traits` like in Rust, but it has another powerful tool: `Partials`!

## Partial struct

```
struct Animal {
    name: str,
    age: i8,

    make_noise: fn(noise?: str) print(noise: noise || "*quiet*")
}

struct Human {
    name: str,
    age: i8,
    job: str

    make_noise: fn() print(noise: "Hello World")
}

type WithName = { name: str };

main {
    // The function will only have access to the name field
    let get_name = fn(thing: WithName) -> str {
        thing.name;
    }

    let me = Human{name: "Google", job: "Programmer", age: 19};
    let some_animal = Animal{name: "Perry", age: 2};

    get_name(me); // Google
    get_name(some_animal); // Perry
}
```

## Partial function

```
// Requires the type to have a "make_noise" method - we don't care about params or return value in this case
type MakesNoise = { make_noise: () };

main {
    let me = Human{name: "Google", job: "Programmer", age: 19};
    let some_animal = Animal{name: "Perry", age: 2};

    let stuff_that_makes_noise = Vec<MakesNoise>{};
    stuff_that_makes_noise.push(me);
    stuff_that_makes_noise.push(some_animal);

    stuff_that_makes_noise.get(0)?.make_noise(); // "Hello World"
    stuff_that_makes_noise.get(1)?.make_noise(); // *quiet*
}

```