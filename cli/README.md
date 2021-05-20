
# Lazy-Cli

To use lazy-cli, first build it via `cargo`:

```
cargo build --release
```

and then run the executable:

```
./target/release/lazy-cli.exe -r main.lazy
```

## Flags and options

```
USAGE:
    lazy-cli.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -t, --time       Shows you the parsing time of the code
    -V, --version    Prints version information

OPTIONS:
    -r, --run <run>    Runs a lazy file
```