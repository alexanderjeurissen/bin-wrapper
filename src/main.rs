use atty::Stream;
use log::Level;
use log::{debug, error, info, log_enabled, trace};
use std::env;
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
  command_args: Vec<String>,
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

// NOTE: create a pipe based on the --stdout / --stderr mode
fn pipe(mode: &Mode) -> Stdio {
  match mode {
    Mode::Proxy => Stdio::inherit(),
    Mode::Capture => match log_enabled!(Level::Debug) {
      true => Stdio::inherit(),
      false => Stdio::null(),
    },
    Mode::CaptureFormachines => {
      if atty::is(Stream::Stdout) {
        Stdio::inherit()
      } else {
        match log_enabled!(Level::Debug) {
          true => Stdio::inherit(),
          false => Stdio::null(),
        }
      }
    }
  }
}

fn main() {
  let start = Instant::now();
  let args = Cli::from_args();
  pretty_env_logger::init_custom_env("LOG");

  trace!("command line options: {:?}", args);

  process_skip_if_env(args.skip_if_env);
  process_resume_if_env(args.resume_if_env);

  let command = &args.command;
  let command_args = &args.command_args.join(" ");

  info!(
    "attempting to run '{}' with args: '{}'",
    command, command_args
  );

  let p_start = Instant::now();

  let mut child = Command::new(&args.command)
    .args(&args.command_args)
    .stdout(pipe(&args.stdout))
    .stderr(pipe(&args.stderr))
    .spawn()
    .expect("failed to execute child process");

  // NOTE: wait for process to finish
  let output = child.wait().expect("failed to wait on child process");

  // NOTE: process has finished, log execution time
  let p_duration = p_start.elapsed();

  // NOTE: we are done log total execution time
  let duration = start.elapsed();

  info!(
    "'{0}' FINISHED after {1:?} (total: {2:?}) exit code: {3:?}",
    command,
    p_duration,
    duration,
    output.code().unwrap()
  );

  std::process::exit(match output.code() {
    Some(code) => code,
    None => panic!(),
  });
}
