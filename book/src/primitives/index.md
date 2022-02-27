
# Primitives

Lazy provides a lot of primitive data types which make using the language a breeze.

## Scalar types

- Signed integers: `i8`, `i16`, `i32`
- Unsigned integers: `u8`, `u16`, `u32`
- Floating points: `f32`, `f64`
- Boolean: `bool` (`true` or `false`)
- Characters: `char` (`'a'`)
- Strings: `str` (`"This is a string."`)

## Compound types

Types which hold more than a single value. None of these types truly **exists** during runtime - they're all abstractions.

### Tuples

Groups multiple different types inside a single structure.

```
let items = (1, 3.14, "Hello World!")
print(items.0, items.1, items.2) // Prints 1 3.14 Hello World!
```

### Lists

A list is a data structure which holds multiple values of the same type. Most of the time, lists are **lazily evaluated**, meaning it's contents don't get
computated and saved in memory until they get requested, which means that this type basically doesn't exist during runtime until you decide to `collect` it into
a different data structure, like a `Vec`.

```
// Create a list with i32 elements, from 1 to 9. 
let my_list = [1, 2, 3, 4, 5, 6, 7, 8, 9] 

// This is a lazy evaluated list. The integers from 1 to 9 don't get stored in memory until requested.
let my_lazy_list = [1..9]

// Filter and map the list lazily. Still nothing has been saved in memory.
let my_edited_lazy_list = my_lazy_list 
                            |> filter x -> x % 2 == 0
                            |> map x -> x * 2

// Now everything is saved in memory, in a vector
let my_vec = my_edited_lazy_list.collect()

print(my_vec == [4, 8, 12, 16])
```

### Records

Records can be boiled down to **named tuples**. They contain **fields**, and each field has a name and a value, which can be of any type. 
Think of records as structs in Rust, and objects in Javascript.

```
let me = {
    name: "Google",
    age: 22
}

print(me.name, me.age) // Google 22

// Passing records in functions
fn use_user(user: { name: str, age: i32 }) {
    print(user.name, user.age)
}

use_user(me)

// This is perfectly valid, this is called structural typing: The function specifies what it needs, 
// and only uses what it needs. You can restrict this behaviour, but more on that later.
use_user({ name: "Google", age: 22, gender: 1 })
```

On their own, records don't do much, but when they're saved in a `type`, then they get a lot more powerful, but more on that later.
