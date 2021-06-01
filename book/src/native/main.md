
# Native modules

Any `dynamic library` can be loaded into a lazy program. 

```

#native("./path/to/dynamic/lib.dll")
struct SymbolsFromNativeLib {
    static add: (num1: i32, num2: i32) -> i32
    static replace: (string: str, pattern: str, new: str) -> str
}

main {
    let num = SymbolsFromNativeLib::add(1, 5); // 6
}
```

File which compiles to `./path/to/dynamic/lib.dll`:

```rs
use lazydyno::DataTypes;

pub fn add(num1: DataTypes::i32, num: DataTypes::i32) -> DataTypes::i32 {
    num1 + num2
}

pub fn replace(string: DataTypes::str, pattern: DataTypes::str, new: DataTypes::str) -> DataTypes::str {
    string.replace(pattern, new)
}
```

## Structs

Native:
```rs
use lazydyno::{Struct, DataTypes};

pub fn create_person(name: DataTypes::str, age: DataTypes::i32) -> Struct {
    let person = Struct::new();
    person.add_str(name);
    person.add_i32(age);
    person
}
```

Lazy:

```
struct Person {
    name: str,
    age: i32
}

#native("...")
struct PersonInterface {
    static create_person: (name: str, age: i32) -> Person
}

main {
    let {name, age} = PersonInterface::create_person("Google", 19); // same as new Person { name: "Google", age: 19 };
    print(name, age); // "Google" 19
}
```

## Enums

Native:
```rs
use lazydyno::{Enum, DataTypes};

pub fn create_result(val: i32) -> Enum::Variant {
    let res = Enum::common::Result::new(); // Result is a built-in enum
    res.set_i32(Enum::common::Result::Ok, val);
    res
}

pub fn create_custom(enum_id: Enum::Id, var_id: Enum::Id, val: DataTypes::i32) -> Enum::Variant {
    let res = Enum::from(enum_id);
    res.variant(var_id);
    res.set_usize(val);
}
```

Lazy:

```
import "std/native" as Native;

enum Sizes {
    S: i32,
    M: i32,
    L: i32,
    XL: i32,
    None
}

#native("...")
struct EnumInterface {
    static create_result(val: i32) -> Result
    static create_custom(enum_id: u16, var_id: u16, val: i32) -> Sizes
}

main {
    let res = EnumInterface::create_custom(Native::idof(Sizes), Native::idof(Sizes::XL), 32);
    print(res == Sizes::XL); // true
}
```