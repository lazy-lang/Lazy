# Lazy

Lazy is a dynamic, interpreted programming language with a manual memory management system. It aims to be performant, light and easy to use and learn.

This document goes over all of Lazy's features one by one.

## Table of content

## Comments

```
// This is a comment on a single line

/*
Multi line comments
*/
```

## Data Types

Every value in Lazy has a specific `data type`. 
Since Lazy is a dynamic programming language, all data types are hidden from the programmer. 

### Bools

Bools can be either `true` or `false`.

### Numbers

The data type for all decimal numbers is the `Number` data type - doesn't matter if the number is real or just an integer.

Lazy supports scientific notation.

### Strings

Strings are arrays of characters, handled by the `String` data type. They are **mutable** by nature. `str += str1` won't create a new string and assign it to `str`, but instead it will append `str1` at the end of `str`.

### Arrays

`Array`s can contain any data type, and can have an unlimited amount of entries. 

```
let arr = [1, 2, 3, "hello!", true, []];
```

### Tables

A `Table` is basically a built-in hashmap. It contains key-value pairs. 

```
let mymap = {a: 1, b: 2};
mymap["a"]; // 1
mymap["c"] = 4;
```

### Enums

Enums are **static** key-value pairs. Their values are constant and cannot be overwritten. Every enum field has a "value", which is by default a number, which gets incremented.

```
enum Names {
    Google,
    SomeoneElse,
    Idk = {}
}

print(Names.Google); // 0
print(Names.SomeoneElse); // 1
print(Names.Idk); // {} (Empty table)
Names.Idk.something = 5; // Invalid
```

### Structs

Structs are very similar to tables, but their keys are known at compile time, which means they are heavily optimized.

```
struct Person {
    name,
    age
}

let person = Person { name: "Google", age: 32 };
person.name; // "Google"
person.something = 3; // Invalid
```

### Empty

By default... everything is `Empty`! Variables / fields in Lazy can either be "full" or "empty". Trying to operate on an empty value will result in a runtime error.

```
let myVal; // Empty!
myVal = Empty; // Still empty...
myVal = 1; // Not empty anymore 
```

There are two main ways to check if a variable is `Empty`:

```
if (myVal == Empty) return;
```

That doesn't look very clean... that's why there is the `?` operator:

```
myVal?; // Exits the entire execution block
```

The `?` operator is also used for **optional chaining**:

```
person.name?.length;
```

Except here it doesn't exit the entire execution block, it just returns `Empty`. You can combine the usage of the `?` operator:

```
let lengthOfName = person.name?.length?;
```

### Functions

In Lazy, functions are **first-class citizens**, they can be assigned to variables, passed as arguments or stored in a data structure.

```
const add = (a, b) => a + b;
add(1, 2); // Returns 3
```

One important detail in Lazy is that functions **do not have access to the outside enviourment**. For example:

```
const main = () => {
    const me = "Google";
    const cb = () => {
        return me.length; // me is not defined???
    }
} 
```

That's when **capturing** comes in - you have to manually specify which variables you want to get captured. Capturing a variables moves it to the enviourment of the function, the function **owns** the variable until it gets out of scope.

```
const cb = |me|() => me.length;
```

## Memory management / ownership

Lazy's memory management system is very similar to Rust's - Every execution block / data structure either **owns** or **references** a location in memory. 

Rules:

- Two execution blocks / objects cannot own the same memory locations.
- There can be an unlimited amount of references to a location in memory, but it cannot be **moved**.

Examples:

```
const setPersonName = (person) => person?.name = "Google";
let me = Person{}; 
setPersonName(me); // The function now owns me
me.name; // Error: object was moved.

// To fix this error we are going to give the function a REFERENCE to the object.

setPersonName(&me); 
me.name; // "Google"!
```

```
const people = {};
people["Google"] = Person{name: "Google"};
people["Moogle"] = Person{name: "Moogle"}; 
// These two are owned by "people", but since you own "people", you also own everything inside it.

const peopleList = |people|() => Array.from(people); // Array from moves the values inside "people", but doesn't care who owns "people"... so now "people" is empty!

people["Google"]; // Error: object was moved.

const peopleArr = peopleList(); // An array of people, owned by us
const anotherPeopleArr = peopleList(); // Returns an empty array, since "people" is empty
```