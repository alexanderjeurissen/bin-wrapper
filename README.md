# bin_wrapper

Simple binstub wrapper written in rust.


### Features:

decorates bin stubs with the great logging / debugging utilities from Rust.

#### Start / End time and duration logging [LOG_LEVEL = debug]
- when RUST_LOG=debug is provided:
 - print start / end time
 - print duration of command

#### StdOut / StdErr redirection

two modes:

- Proxy => simply pass-through the stdout and stderr (default behaviour)
- Capture => consume stdout and stderr and print it to logs
  - prints `stdout` to TRACE log level
  - prints `stderr` to DEBUG log level

#### Command line options

```
bin-wrapper 0.1.0
Explanation of bin-wrapper usage.

USAGE:
    bin-wrapper [OPTIONS] <command> [args]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --mode <mode>     [default: Proxy]  [possible values: Capture, Proxy]

ARGS:
    <command>
    <args>...
```
