use crate::paths::PATHS;

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{fs::File, io::prelude::*};

pub const SPLITTER: usize = 20;

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn log(message: &str, console: bool) {
    LOGGER.lock().unwrap().log(message, console);
}

pub fn log_inline(message: &str, console: bool) {
    LOGGER.lock().unwrap().log_inline(message, console);
}

pub fn section(title: &str, console: bool) {
    log(&format!("{:-^1$}", title, 60), console);
}

pub fn subsection(title: &str, console: bool) {
    log(&format!("[{}]", title), console);
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
        self.log_inline(&format!("{}\r\n", message), console);
    }

    pub fn log_inline(&mut self, message: &str, console: bool) {
        if console {
            print!("{}", message);
        }

        #[cfg(feature = "logging")]
        {
            if !self.enabled {
                return;
            }

            if let Some(file) = &mut self.file {
                _ = write!(file, "{}", message);
            }
        }
    }
}
