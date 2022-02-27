
# Lists, iterators and vectors

## Lists

As we mentioned earlier, lists contain multiple data of the same type, and they can hold potentially infinite data, because the data is not allocated unless it is
requested. 

You can concat, repeat, or partition lists **immutably** (a new list gets created) via the `+`, `*` and `/` operators:

```
[1, 2, 3] + [1, 2, 3, 4, 5] // [1, 2, 3, 1, 2, 3, 4, 5]
[1, 2, 3] * 3 // [1, 2, 3, 1, 2, 3, 1, 2, 3]
[1, 2, 3, 4] / 2 // [[1, 2], [3, 4]]
```

But how do we request data? You could either `collect` it, which will put it all in a vector:

```
let my_list = [1, 2, 3, 4, 5, 6, 7, 8, 9]
                |> filter x -> x > 5
                |> collect
```

or you could use `next` to get a single element out:

```
let my_list = [1, 2, 3, 4, 5]

let first_element = my_list.next()?
let second_element = my_list.next()?
```

Be careful with `next`, it **progresses** the list. Once `next` is called, the first element gets collected only, and the next element in the list becomes the first.
This happens because the list is a general interface of the `Iterator` type.

## Iterators

Iterators are an **abstraction** over iteration. An iterator is basically a structure which holds a **state** which gets changed every time the `next` function is called on it.
The generalized type for an iterator is: 

```
type Iterator(T) = {
    impl next() -> T
}
```

Any type which implements the `next` function can potentially be an iterator. 

### Iterator literals

Iterator literals are just syntax - they don't represent any actual value. Different language constructs may use these literals for different things.

```
let my_list = [1..10] // 1 to 10
```

All iterator literals are **inclusive**. For example, the above literal will start from 1 and end at exactly 10 (not 9, like in Rust for example). The end range may be excluded
from an iterator, that means it's infinite, or well, `next` will return a value intil the max signed / unsigned integer gets hit.

Iterators can also represent a range between characters: `'a'..'z'`.

Literals may be used in `for...in` loops:

```
for x in -100..100 {
    print(x)
}
```

Or in `match` expressions:

```
match x {
    0..5 => print("Number is between 0 and 5."),
    _ => print("Number is not between 0 and 5.)
}
```

And of course, in list expressions:

```
let my_list = [1..5000]
```

### Vectors

A vector is a contiguous growabale list - you can modify it any time by pushing, or removing elements into it. All elements inside the vector are stored in the heap. You can
think of a vector as a mutable mutable list.

```
let vec = [1, 2, 3, 4, 5, 6, 7, 8, 9]
            |> collect

let vec2 = mut [1, 2, 3, 4, 5, 6, 7, 8, 9]
            |> Vec.from

print(vec == vec2) // true

// Important: In order to modify the vector, you need to add the mut keyword before it
vec2.push(10)

print(vec == vec2) // false
```






