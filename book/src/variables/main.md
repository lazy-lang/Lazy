
# Variables

Defining variables is done with the `let`, `const` and `static` keywords.

## let

`let` defines a **mutable** variable. The value inside the variable itself is **always** mutable, but the **variable** can be changed.

```
let a = 5;
a = 10; // Valid
a = 3.14; // Invalid, `a` is implicitly an i32
```

## const

`const` defines an **immutable** variable. The value is still **mutable**, but the **variable** cannot be changed.

```
const a = 5;
a = 10; // Invalid
```

```
struct Pair<V> {
    key: str,
    val: V
}

const my_pair = new Pair<i32>{key: "Hello", val: 3};
my_pair.key = "World"; // Valid
```

## static

`static` is used for values you **never** want to get garbage-collected. They cannot be defined in functions or the `main` block.

```
static PI = 3.14;

main {
    PI = 4; // Invalid
    let PI = 4; // Invalid
    const PI = 5; // Invalid
}
```


## Type hinting in variables

```
let someOption = none;  // Incorrect
```

Here, the compiler doesn't know what else can `someOption` be other than none, so it throws an error. You need to specify the other possible type.

```
let someOption: i32? = none; // Correct!
```

## Deconstructing structs or tuples

```
let my_tuple = [1, 2, 3, 4, 5];
let [firstElement, secondElement] = my_tuple;
print(firstElement, secondElement); // 1, 2
```

```
struct Student {
    grades: Map<str, Grade>
    favorite_subject: str
}

const { favorite_subject } = new Student { grades: Map::create(), favorite_subject: "programming" };
print(favorite_subject); // "programming"
```