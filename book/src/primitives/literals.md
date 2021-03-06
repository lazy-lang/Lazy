
# Literals

Almost all literals are what'd you expect them to be.

## Numbers

```
let my_age = 18; //  inferred type: i32
let my_height = 6.1; // inferred type: f32

let not_inferred: u8 = 12; // Not inferred, number is an u8
```

Numbers can contain underscores (`_`) to improve readability:

```
let big_num = 100_00_00; 
let same_big_num = 1000000;
big_num == same_big_num; // returns "true"
```

To improve working with milliseconds, numbers can be prefixed with `s`, `m`, `h` and `d` to be automatically converted. This is done during compile time, so there's zero overhead!

```
let one_min_in_milliseconds = 1m; 
one_min_in_milliseconds == 60000; // returns "true"
```

### Binary / Octal / Hex literals

Numbers can be prefixed with `0b`, `0o` and `0x` to be parsed as binary / octal / hex numbers.

```
0b010101 == 21 // true
0o03621623 == 992147 // true
0x0123ABCd == 19114957 // true
```

## Template literals

String concatenation is very messy in many languages - Lazy makes this easy with template literals:

```
let age = 1 + 17;
print(`Hello World! I am ${age} years old`); // Hello World! I am 18 years old
```

They work exactly like in javascript!

## Iterators

```
0..5; // Creates an iterator from 0 (inclusive) to 5 (exclusive), so 0, 1, 2, 3 and 4.
5..=10; // Creates an iterator from 5 (inclusive) to 10 (inclusive) 
..5; // Short syntax for 0..5
..=10; // Short syntax for 0..=10
```

## Functions

In `Lazy`, functions are **first-class citizens**, you can pass them as parameters, save them in variables and so on.

```
let my_fn = fn() 1 + 1;
my_fn(); // returns 2
```

## Natural literals

`natural` literals are `tuples` or `iterators` which can only contain literals. For example:

```
person.age..=18; // This is NOT a natural iterator
0..=18; // This IS, because both the start and the end are literals

[1, 2, none, true] // This IS a natural tuple
[1, 2, func(1 + 1)] // This is NOT a natural tuple
```

Only natural `tuples` or `iterators` are allowed in `match` arms.