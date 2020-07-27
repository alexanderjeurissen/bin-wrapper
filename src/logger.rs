use slog::{o, trace, Drain, Level, Logger};
use slog_term::{CompactFormat, TermDecorator};

// NOTE: Based off https://github.com/ragone/lint-emit/blob/master/src/logger.rs
// Get the logger based on verbosity
pub fn setup_logger(verbosity: u64) -> Logger {
  // Setup logging level
  let min_log_level = match verbosity {
    0 => Level::Critical,
    1 => Level::Debug,
    _ => Level::Trace,
  };

  // Create logger
  let decorator = TermDecorator::new().build();
  let drain = CompactFormat::new(decorator).build().fuse();
  let drain = slog_async::Async::new(drain).build().fuse();
  let logger = Logger::root(drain.filter_level(min_log_level).fuse(), o!());
  trace!(logger, "{:#?} logging enabled", min_log_level);
  logger
}
