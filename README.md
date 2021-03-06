# bin_wrapper

Simple binstub wrapper written in rust.
Aims to extract common features in bin_stubs so bin_stubs can focus on the actual business logic.

![image](https://github.com/alexanderjeurissen/bin-wrapper/blob/main/public/output_redirection.png)

### Features:

decorates bin stubs with the great logging / debugging utilities from Rust.

#### Start / End time and duration logging [LOG_LEVEL = debug]
Print out start/end time as well as duration of the sub command that is executed.

#### StdOut / StdErr redirection

Nested commands can be very verbose. Instead of dealing with a huge variance of verbosity flags, let bin-wrapper capture all noisy output and print out the content you care about.

two options `--stdout` and `--stderr` can be configured.

three modes are supported:
- `Proxy` => simply pass-through the stdout and stderr (default behaviour)
- `Capture` => consume stdout and stderr and print it to logs
  - prints `stdout` to TRACE log level
  - prints `stderr` to DEBUG log level
- `CaptureForMachines` => consume stdout or stderr and print it to logs if the script is run by a machine (aka not a terminal).

#### Env variable guards

Bin-wrapper provides the option to define env variable guards.
This removes the need for bin-stubs like this:

```sh

[ -z $SOME_ENV_VAR ] || [ $SOME_ENV_VAR -eq 0 ] && exit 0

yarn some_command
```

two options are provided:

 - skip_if_env: cancel execution if env variable is set
 - resume_if_env resume execution if env variable is set


### Command line options

```
bin-wrapper 1.0.0
Explanation of bin-wrapper usage.

USAGE:
    bin-wrapper [OPTIONS] <command> [args]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --resume-if-env <resume-if-env>    Lookup the provided ENV variable and only resume execution if set
        --skip-if-env <skip-if-env>        Lookup the provided ENV variable and skip execution if set
        --stderr <stderr>                  How should bin-wrapper redirect stderr ? [default: Proxy]  [possible values:
                                           Capture, Proxy, CaptureFormachines]
        --stdout <stdout>                  How should bin-wrapper redirect stdout ? [default: Proxy]  [possible values:
                                           Capture, Proxy, CaptureFormachines]

ARGS:
    <command>
    <args>...
```
