use atty::Stream;
use log::{error, info, trace};
use std::env;
use std::io::{self, Result, Write};
use std::process::{exit, Command, Stdio};
use std::time::Instant;
use structopt::clap::arg_enum;
use structopt::StructOpt;

extern crate pretty_env_logger;

arg_enum! {
    #[derive(Debug)]
    enum Mode {
      Capture,
      Proxy,
      CaptureFormachines
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "bin-wrapper", about = "Explanation of bin-wrapper usage.")]
struct Cli {
  #[structopt(long, possible_values = &Mode::variants(), case_insensitive = true, default_value = "Proxy", help = "How should bin-wrapper redirect stdout ?")]
  stdout: Mode,

  #[structopt(long, possible_values = &Mode::variants(), case_insensitive = true, default_value = "Proxy", help = "How should bin-wrapper redirect stderr ?")]
  stderr: Mode,

  #[structopt(
    long,
    help = "Lookup the provided ENV variable and skip execution if set"
  )]
  skip_if_env: Option<String>,

  #[structopt(
    long,
    help = "Lookup the provided ENV variable and only resume execution if set"
  )]
  resume_if_env: Option<String>,

  command: String,
  args: Vec<String>,
}

fn set_log_level() {
  // NOTE: ensure default log level is NONE
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "none")
  }
}

fn process_skip_if_env(skip_if_env: Option<String>) {
  match skip_if_env {
    Some(x) => {
      if env::var(x).is_ok() {
        trace!("*skip_if_env* present, ending execution");
        exit(0)
      } else {
        trace!("*skip_if_env* not set, resuming execution");
      }
    }
    None => trace!("*skip_if_env* not present, resuming execution"),
  }
}

fn process_resume_if_env(resume_if_env: Option<String>) {
  match resume_if_env {
    Some(x) => {
      if env::var(x).is_err() {
        trace!("*resume_if_env* not set, ending execution");
        exit(0)
      } else {
        trace!("*resume_if_env* present, resuming execution");
      }
    }
    None => trace!("*resume_if_env* not present, resuming execution"),
  }
}

fn redirect_std_out(stdout: Vec<u8>, mode: Mode) {
  match mode {
    Mode::Proxy => {
      io::stdout().write_all(&stdout).expect("cant proxy stdOut");
    }
    Mode::Capture => {
      let raw_output = String::from_utf8(stdout).unwrap();

      raw_output.lines().for_each(|x| trace!("{}", x));
    }
    Mode::CaptureFormachines => {
      if atty::is(Stream::Stdout) {
        io::stdout().write_all(&stdout).expect("cant proxy stdOut");
      } else {
        let raw_output = String::from_utf8(stdout).unwrap();

        raw_output.lines().for_each(|x| trace!("{}", x));
      }
    }
  }
}

fn redirect_std_err(stderr: Vec<u8>, mode: Mode) {
  match mode {
    Mode::Proxy => {
      io::stderr().write_all(&stderr).expect("cant proxy stdErr");
    }
    Mode::Capture => {
      let raw_output = String::from_utf8(stderr).unwrap();

      raw_output.lines().for_each(|x| error!("{}", x));
    }
    Mode::CaptureFormachines => {
      if atty::is(Stream::Stderr) {
        io::stderr().write_all(&stderr).expect("cant proxy stdOut");
      } else {
        let raw_output = String::from_utf8(stderr).unwrap();

        raw_output.lines().for_each(|x| error!("{}", x));
      }
    }
  }
}

fn main() -> Result<()> {
  set_log_level();

  pretty_env_logger::init_custom_env("LOG");

  let args = Cli::from_args();

  trace!("command line options: {:?}", args);

  process_skip_if_env(args.skip_if_env);
  process_resume_if_env(args.resume_if_env);

  let command = args.command;
  let command_args = args.args.join(" ");

  info!(
    "attempting to run '{}' with args '{}'",
    command, command_args
  );

  let start = Instant::now();

  let child = Command::new(&command)
    .arg(&command_args)
    .stdin(Stdio::inherit())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  let output = child.wait_with_output()?;

  let duration = start.elapsed();

  redirect_std_out(output.stdout, args.stdout);
  redirect_std_err(output.stderr, args.stderr);

  if output.status.success() {
    info!(
      "'{0}' finished after {1:?} with exit code: {2:?}",
      command, duration, output.status
    );
    exit(0)
  } else {
    error!(
      "'{0}' failed after {1:?} with exit code {2:?}",
      command, duration, output.status
    );
    exit(1)
  }
}
