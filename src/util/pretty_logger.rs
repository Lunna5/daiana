//! Formatted console logger with emojis, colors, and aligned module targets.

use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use env_logger::Builder;
use log::Level;

/// Initializes the pretty logger using `RUST_LOG` environment variable filter.
pub fn init() {
    try_init().unwrap();
}

/// Initializes the pretty logger with millisecond timestamps enabled.
pub fn init_timed() {
    try_init_timed().unwrap();
}

/// Attempts to initialize the pretty logger using the `RUST_LOG` environment variable.
pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

/// Attempts to initialize the timed pretty logger using the `RUST_LOG` environment variable.
pub fn try_init_timed() -> Result<(), log::SetLoggerError> {
    try_init_timed_custom_env("RUST_LOG")
}

/// Initializes the logger reading filter directives from a custom environment variable name.
pub fn init_custom_env(environment_variable_name: &str) {
    try_init_custom_env(environment_variable_name).unwrap();
}

/// Attempts to initialize the logger reading filter directives from a custom environment variable name.
pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder();

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse_filters(&s);
    }

    builder.try_init()
}

/// Attempts to initialize the timed logger reading filter directives from a custom environment variable name.
pub fn try_init_timed_custom_env(
    environment_variable_name: &str,
) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_timed_builder();

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse_filters(&s);
    }

    builder.try_init()
}

/// Creates a new [`Builder`] pre-configured with emoji level formatting and aligned target padding.
pub fn formatted_builder() -> Builder {
    let mut builder = Builder::new();

    builder.format(|f, record| {
        use std::io::Write;

        let target = record.target();
        let max_width = max_target_width(target);
        let level = colored_level(record.level());

        let target_padded = BoldPadded {
            value: target,
            width: max_width,
        };

        writeln!(f, " {} │ {} ❯ {}", level, target_padded, record.args())
    });

    builder
}

/// Creates a new [`Builder`] pre-configured with millisecond timestamps, emoji levels, and aligned targets.
pub fn formatted_timed_builder() -> Builder {
    let mut builder = Builder::new();

    builder.format(|f, record| {
        use std::io::Write;
        let target = record.target();
        let max_width = max_target_width(target);
        let level = colored_level(record.level());

        let target_padded = BoldPadded {
            value: target,
            width: max_width,
        };

        let time = f.timestamp_millis();

        writeln!(
            f,
            " {} {} │ {} ❯ {}",
            time,
            level,
            target_padded,
            record.args()
        )
    });

    builder
}

struct BoldPadded<'a> {
    value: &'a str,
    width: usize,
}

impl<'a> fmt::Display for BoldPadded<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\x1b[35m{: <width$}\x1b[0m",
            self.value,
            width = self.width
        )
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level(level: Level) -> &'static str {
    match level {
        Level::Trace => "\x1b[35m🌸 TRACE\x1b[0m",
        Level::Debug => "\x1b[36m🦋 DEBUG\x1b[0m",
        Level::Info => "\x1b[32m🌱 INFO \x1b[0m",
        Level::Warn => "\x1b[33m✨ WARN \x1b[0m",
        Level::Error => "\x1b[31m🍓 ERROR\x1b[0m",
    }
}
