extern crate slog;
extern crate slog_async;
extern crate slog_term;

use atty::Stream;
use logger::*;
use slog::{debug, error, info, o, trace};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::sync::{Arc, Barrier};
use std::thread;
use threadpool::ThreadPool;

use std::time::Instant;
use structopt::clap::arg_enum;
use structopt::StructOpt;
use subprocess::{Popen, PopenConfig, Redirection};

mod logger;

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

  #[structopt(
    short,
    long,
    parse(from_occurrences),
    help = "Control the output verbosity"
  )]
  verbose: u64,

  command: Vec<String>,
}

fn process_skip_if_env(skip_if_env: Option<String>, logger: &slog::Logger) {
  match skip_if_env {
    Some(x) => {
      if env::var(x).is_ok() {
        trace!(logger, "*skip_if_env* present, ending execution");
        exit(0)
      } else {
        trace!(logger, "*skip_if_env* not set, resuming execution");
      }
    }
    None => trace!(logger, "*skip_if_env* not present, resuming execution"),
  }
}

fn process_resume_if_env(resume_if_env: Option<String>, logger: &slog::Logger) {
  match resume_if_env {
    Some(x) => {
      if env::var(x).is_err() {
        trace!(logger, "*resume_if_env* not set, ending execution");
        exit(0)
      } else {
        trace!(logger, "*resume_if_env* present, resuming execution");
      }
    }
    None => trace!(logger, "*resume_if_env* not present, resuming execution"),
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
enum CaptureDataType {
  Stdout,
  Stderr,
}
fn capture(file: File, capture_data: CaptureDataType, logger: slog::Logger) {
  let log = logger.clone();
  let reader = BufReader::new(file);

  reader.lines().for_each(|line| match capture_data {
    CaptureDataType::Stdout => info!(log, "{}", line.unwrap()),
    CaptureDataType::Stderr => error!(log, "{}", line.unwrap()),
  });
}

fn main() {
  let start = Instant::now();
  let args = Cli::from_args();
  let logger = setup_logger(args.verbose);

  trace!(logger, "command line options: {:?}", args);

  process_skip_if_env(args.skip_if_env, &logger);
  process_resume_if_env(args.resume_if_env, &logger);

  let command = &args.command.join(" ");

  debug!(logger, "attempting to run '{}'", command);

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

  // NOTE: instantiate threadpool with two workers
  let n_workers = 2;
  let pool = ThreadPool::with_name("runner".into(), n_workers);
  // NOTE: create a barrier that waits for all jobs plus the starter thread
  let barrier = Arc::new(Barrier::new(n_workers + 1));

  if let Redirection::Pipe = pipe(&args.stdout) {
    let stdout = p.stdout.take().unwrap();
    let log = logger.new(o!("stdout" => ""));
    let stdout_barrier = barrier.clone();

    pool.execute(move || {
      capture(stdout, CaptureDataType::Stdout, log);

      // NOTE:wait for the other threads
      stdout_barrier.wait();
    });
  }

  if let Redirection::Pipe = pipe(&args.stderr) {
    let stderr = p.stderr.take().unwrap();
    let log = logger.new(o!("stderr" => ""));
    let stderr_barrier = barrier.clone();

    pool.execute(move || {
      capture(stderr, CaptureDataType::Stderr, log);

      // NOTE:wait for the other threads
      stderr_barrier.wait();
    });
  };

  // NOTE: wait for process to finish
  p.wait().unwrap();

  // NOTE: process has finished, log execution time
  let p_duration = p_start.elapsed();

  // NOTE: wait for workers in threadpool to finish
  barrier.wait();

  // NOTE: we are done log total execution time
  let duration = start.elapsed();

  debug!(
    logger,
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
