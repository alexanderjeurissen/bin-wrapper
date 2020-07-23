# bin_wrapper

Simple binstub wrapper written in rust.


### Features:

- decorates bin stubs with the great logging / debugging utilities from Rust.
- when RUST_LOG=debug is provided:
 - print start / end time
 - print duration of command

### WIP

- Add option to redirect stderr / stdout of the wrapped command to either traces, or dedicated output files
