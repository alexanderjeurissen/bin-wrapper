use structopt::StructOpt;
use structopt::clap::arg_enum;
use std::process::{Command, Stdio, exit};
use std::time::{Instant};
use std::io::{self, Write, Result};
use std::env;
use log::{info, trace, warn, error};

extern crate pretty_env_logger;

arg_enum! {
    #[derive(Debug)]
    enum Mode {
        Capture,
        Proxy
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "bin-wrapper", about = "Explanation of bin-wrapper usage.")]
struct Cli {
  #[structopt(long, possible_values = &Mode::variants(), case_insensitive = true, default_value = "Proxy", help = "How should bin-wrapper redirect stdout/stderr ?")]
  mode: Mode,

  #[structopt(long, help = "Lookup the provided ENV variable and skip execution if set")]
  skip_if_env: Option<String>,

  #[structopt(long, help = "Lookup the provided ENV variable and only resume execution if set")]
  resume_if_env: Option<String>,

  command: String,
  args: Vec<String>,
}

fn main() -> Result<()> {
  // NOTE: ensure default log level is NONE
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "none")
  }

  pretty_env_logger::init();

  let args = Cli::from_args();

  trace!("command line options: {:?}", args);

  match args.skip_if_env {
    Some(x) => {
      if env::var(x).is_ok() {
        warn!("*skip_if_env* present, ending execution");
        exit(0)
      } else {
        info!("*skip_if_env* not set, resuming execution");
      }
    },
    None => trace!("*skip_if_env* not present, resuming execution")
  }

  match args.resume_if_env {
    Some(x) => {
      if env::var(x).is_err() {
        info!("*resume_if_env* not set, ending execution");
        exit(0)
      } else {
        warn!("*resume_if_env* present, resuming execution");
      }
    },
    None => trace!("*resume_if_env* not present, resuming execution")
  }

  let command = args.command;
  let command_args  = args.args.join(" ");

  info!("attempting to run '{}'", command);

  let start = Instant::now();

  let child = Command::new(&command)
    .arg(&command_args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let output = child.wait_with_output()?;

  let duration = start.elapsed();

  match args.mode {
    Mode::Proxy => {
      io::stdout().write_all(&output.stdout).expect("cant proxy StdOut");
      io::stderr().write_all(&output.stderr).expect("cant proxy stdErr");
    },
    Mode::Capture => {
      let raw_std_out = String::from_utf8(output.stdout).unwrap();

      raw_std_out
        .lines()
        .for_each(|x| trace!("{}", x));

      let raw_std_err = String::from_utf8(output.stderr).unwrap();

      raw_std_err
        .lines()
        .for_each(|x| error!("{}", x));
    }
  }

  if output.status.success() {
    info!("'{0}' finished after {1:?} with exit code: {2:?}", command, duration, output.status);
    exit(0)
  } else {
    error!("'{0}' failed after {1:?} with exit code {2:?}", command, duration, output.status);
    exit(1)
  }
}
