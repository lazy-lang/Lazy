
# Iterators

`Iterator` is a type which allows you to loop over a struct. `for...in` loops are powered by iterators. An `Iterator` is any struct which satisfies the following type:

```
type Iterator<T> = { next: () -> T? }
```

Any struct with a "next" function is a valid iterator!


```
struct RangeIter {
    min: i32
    max: i32
    progress: i32

    next: () -> i32? {
        if self.progress == self.max none 
        else self.progress += 1
    }
}

main {
    let my_range = new Range { min: 0, max: 15, progress: 0 };
    for i in my_range print(i);
}
```


## Literals

Iterator literals are called `range` iterators, because they create an iterator that iterates through a range of numbers.

```
0..5 // Exclusive
0..=5 // Inclusive
```

## Spreading iterators

The `spread` syntax can be used to turn iterators into vectors!

```
let iterator = 0..10;
let vec = ...iterator;
print(vec.to_string()); // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
```

### Spread syntax in functions

The `spread` syntax has a special meaning in function parameters:

```
type Stringable = { to_string: () -> str };

const print = fn(...strs: Stringable?) {
    // code
}
```

The `...strs:` syntax means that the function can receive an unlimited amount of values with the `Stringable` type. The `strs` value itself is either a `Vector` which contains `Stringable`, or `none`..

```
const println = fn(...strs: Stringable?) {
    for string in strs {
        print(string?.to_string() + "\n");
    }
}

print_many("Hello", new Vec{}, 4, true); 
// Prints:
/*
Hello
[]
4,
true
*/
```