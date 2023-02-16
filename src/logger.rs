use crate::paths::PATHS;

use lazy_static::lazy_static;
use std::fmt::Display;
#[allow(unused)]
use std::{fs::File, io::prelude::*, sync::Mutex};

pub const SPLITTER: usize = 20;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn log<S: AsRef<str>>(message: S, console: bool)
where
    S: Display,
{
    LOGGER.lock().unwrap().log(message.as_ref(), console);
}

pub fn log_inline<S: AsRef<str>>(message: S, console: bool)
where
    S: Display,
{
    LOGGER.lock().unwrap().log_inline(message.as_ref(), console);
}

pub fn section<S: AsRef<str>>(title: S, console: bool)
where
    S: Display,
{
    log(format!("{:-^1$}", title, 60), console);
}

pub fn subsection<S: AsRef<str>>(title: S, console: bool)
where
    S: Display,
{
    log(format!("[{title}]"), console);
}

#[allow(unused)]
pub struct Logger {
    pub enabled: bool,
    file: Option<File>,
}

impl Logger {
    pub fn new() -> Logger {
        Self {
            enabled: false,
            file: match File::create(&PATHS.log) {
                Err(e) => {
                    eprintln!("failed to open log file {}: {}", PATHS.log, e);
                    None
                }
                Ok(f) => Some(f),
            },
        }
    }

    pub fn log(&mut self, message: &str, console: bool) {
        self.log_inline(&format!("{message}\r\n"), console);
    }

    pub fn log_inline(&mut self, message: &str, console: bool) {
        if console {
            print!("{message}");
        }

        #[cfg(feature = "logging")]
        {
            if !self.enabled {
                return;
            }

            if let Some(file) = &mut self.file {
                _ = write!(file, "{message}");
            }
        }
    }
}
