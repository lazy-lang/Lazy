
# Promises

Async programming in `Lazy` is made to be simple and easy:

```
import "std/http" as Http

main {
    const res = await? Http::get("https://google.com");
    print(res.body);
}
```

## Creating promises

```
let prom = Promise::spawn(fn() -> Result<i32, none> Result::Ok(1 + 1));
let res = await? prom; // 2
```

Promises return a `Result`, which is a special enum with two possible variants - `T`, the actual return value of the promise, or an `Error`:

```
Promise::spawn(fn() -> Result<none, str> {
    Result::Error("This promise always returns an error :x");
});
```

A question mark (`?`) can be put after the `await` keyword to short-circuit the execution context if the promise returns an `Error`.


## Under the hood

Under the hood, `Promise`s are handled by the [tokio](https://tokio.rs/) runtime.

## Timers

### Repeating a function

```
let counter = 0;
Promise::interval(fn() {
    print(counter);
    counter += 1;
}, 1m);
```

### Timeout

```
Promise::timeout(fn() {
    print("This will be executed in 5 seconds!");
}, 5s);
```

### Stopping function execution

```
await Promise::block(5m);
```