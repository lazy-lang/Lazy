# Welcome

This book acts as the **specification** for the **Lazy** programming language! Every detail about the language (grammar, data types, memory management, JIT) is provided, as well as examples. 

Lazy is a statically typed, interpreted programming language. 

## A taste

```
struct Person {
    name: str,
    age: float,
    hobby?: str,

    age_up: fn() {
        self.age += 1;
    }

}

enum AnimalTypes {
    Dog,
    Cat,
    Fish,
    Bird
}

struct Animal {
    name: str,
    age: i8,
    type: AnimalTypes
}

type ThingWithName = {name: str};

main {
    let me = Person{ name: "Google", age: 19 };
    let my_pet = Animal { name: "Baby", age: 1, type: AnimalTypes:Bird };

    let things = Vec<ThingWithName>{};
    things.push(me);
    things.push(my_pet);

    for thing in things {
        print(thing.name);
    }
}
```