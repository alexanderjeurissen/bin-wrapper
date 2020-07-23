use structopt::*;
use std::process::{Command, Stdio, exit};
use std::time::{Instant};
use log::{info, warn};

extern crate pretty_env_logger;

#[derive(Debug, StructOpt)]
struct Cli {
    command: String,
    args: String,
}

fn main() {
  pretty_env_logger::init();
  let args = Cli::from_args();

  let command = args.command.clone();
  let command_args  = args.args.clone();

  info!("attempting to run '{0}' with args: '{1}'", command, command_args);

  let start = Instant::now();
  let status = Command::new(args.command)
                       .arg(args.args)
                       .stdout(Stdio::inherit())
                       .stderr(Stdio::inherit())
                       .status()
                       .expect("failed to execute process");


  let duration = start.elapsed();

  if status.success() {
    info!("'{0}' finished after {1:?} with exit code: {2:?}", command, duration, status);
    exit(0)
  } else {
    warn!("'{0}' failed after {1:?} with exit code {2:?}", command, duration, status);
    exit(1)
  }
}
