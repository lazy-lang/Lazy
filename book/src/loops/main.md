
# Loops

Loops are an imperative way to execute an **expression** multiple times. `Lazy` provides two way to create an imperative loop: `for` and `while`.


## For

In `Lazy`, the for loop only has one variant - `for...in`:

```
for i in 0..10 {
    print(i);
}
// Prints 0, then 1, then 2... up to 9
```

The expression after `in` can be any valid iterator - and an iterator satisfies the following type:

```
type Iterator<T> = { next: () -> T? } // Any struct with a "next" function is a valid iterator!
```

## While

A traditional `while` loop. The expression gets executed as long as the condition is `true`.

```
while true {
    // A never ending loop!
}
```

## Breaking a loop

Both types of loops are expressions... but what do they return? By default, `none`, unless you use the `yield` keyword inside them. The `yield` keyword stops the execution of the loop and returns the provided expression.

The following snippet checks if `20` is in `vector` and if it is, the `value` variable is set to `true`, otherwise it's `none`. 

```
let vector = new Vec<i32>{ iter: 0..30 };

let value = for i in vector {
    if i == 20 yield true
}

if value print("20 is in the vector!")
else print("20 is NOT in the vector!")
```