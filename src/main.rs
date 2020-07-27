use atty::Stream;
use log::{debug, error, info, trace};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::thread;
use std::time::Instant;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use subprocess::{Popen, PopenConfig, Redirection};

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

  command: Vec<String>,
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
fn pipe(mode: &Mode) -> Redirection {
  match mode {
    Mode::Proxy => Redirection::None,
    Mode::Capture => Redirection::Pipe,
    Mode::CaptureFormachines => {
      if atty::is(Stream::Stdout) {
        Redirection::None
      } else {
        Redirection::Pipe
      }
    }
  }
}

// NOTE: create a thread to capture and buffer the output of p.stdout / p.stderr
enum Logger {
  Stdout,
  Stderr,
}
fn capture(file: File, logger: Logger) -> thread::JoinHandle<()> {
  return thread::spawn(move || {
    let reader = BufReader::new(file);

    reader.lines().for_each(|line| match logger {
      Logger::Stdout => info!("{}", line.unwrap()),
      Logger::Stderr => error!("{}", line.unwrap()),
    });
  });
}

fn main() {
  let start = Instant::now();
  let args = Cli::from_args();
  pretty_env_logger::init_custom_env("LOG");

  trace!("command line options: {:?}", args);

  process_skip_if_env(args.skip_if_env);
  process_resume_if_env(args.resume_if_env);

  let command = &args.command.join(" ");

  debug!("attempting to run '{}'", command);

  let p_start = Instant::now();

  let mut p = Popen::create(
    &args.command,
    PopenConfig {
      stdout: pipe(&args.stdout),
      stderr: pipe(&args.stderr),
      ..Default::default()
    },
  )
  .unwrap();

  let mut out_handle: Option<thread::JoinHandle<()>> = None;
  if let Redirection::Pipe = pipe(&args.stdout) {
    let stdout = p.stdout.take().unwrap();
    out_handle = Some(capture(stdout, Logger::Stdout));
  }

  let mut err_handle: Option<thread::JoinHandle<()>> = None;
  if let Redirection::Pipe = pipe(&args.stderr) {
    let stderr = p.stderr.take().unwrap();
    err_handle = Some(capture(stderr, Logger::Stderr));
  };

  // NOTE: wait for process to finish
  p.wait().unwrap();

  // NOTE: process has finished, log execution time
  let p_duration = p_start.elapsed();

  match out_handle {
    Some(v) => v.join().unwrap(),
    None => trace!("stdout logger thread not present, no need to wait."),
  }

  match err_handle {
    Some(v) => v.join().unwrap(),
    None => trace!("stderr logger thread not present, no need to wait."),
  }

  // NOTE: we are done log total execution time
  let duration = start.elapsed();

  debug!(
    "'{0}' FINISHED after {1:?} (total: {2:?}) exit code: {3:?}",
    command,
    p_duration,
    duration,
    p.exit_status()
  );

  std::process::exit(match p.exit_status().unwrap() {
    subprocess::ExitStatus::Exited(a) => a as i32,
    _ => panic!(),
  });
}
