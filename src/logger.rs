use crate::paths::PATHS;

use lazy_static::lazy_static;
use std::fmt::Display;
use std::{fs::File, io::prelude::*, sync::Mutex};

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn set_is_first_section(first: bool) {
    LOGGER.lock().unwrap().is_first_section = first;
}

pub fn get_is_first_section() -> bool {
    LOGGER.lock().unwrap().is_first_section
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
    if get_is_first_section() {
        set_is_first_section(false);
    } else {
        log("", console);
    }

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
    pub is_first_section: bool,
    file: Option<File>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            enabled: false,
            is_first_section: true,
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
