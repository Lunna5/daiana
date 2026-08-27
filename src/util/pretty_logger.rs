use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use env_logger::Builder;
use log::Level;

pub fn init() {
    try_init().unwrap();
}

pub fn init_timed() {
    try_init_timed().unwrap();
}

pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

pub fn try_init_timed() -> Result<(), log::SetLoggerError> {
    try_init_timed_custom_env("RUST_LOG")
}

pub fn init_custom_env(environment_variable_name: &str) {
    try_init_custom_env(environment_variable_name).unwrap();
}

pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder();

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse_filters(&s);
    }

    builder.try_init()
}

pub fn try_init_timed_custom_env(
    environment_variable_name: &str,
) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_timed_builder();

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse_filters(&s);
    }

    builder.try_init()
}

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

        writeln!(f, " {} {} │ {} ❯ {}", time, level, target_padded, record.args())
    });

    builder
}

struct BoldPadded<'a> {
    value: &'a str,
    width: usize,
}

impl<'a> fmt::Display for BoldPadded<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1b[35m{: <width$}\x1b[0m", self.value, width = self.width)
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