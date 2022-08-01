use crate::paths::PATHS;

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{fs::File, io::prelude::*};

lazy_static! {
    pub static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

pub fn set_enabled(enabled: bool) {
    LOGGER.lock().unwrap().enabled = enabled;
}

pub fn log(message: &str) {
    LOGGER.lock().unwrap().log(message);
}

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
                    eprintln!("couldn't open {}: {}", PATHS.log, e);
                    None
                }
                Ok(f) => Some(f),
            },
        }
    }

    pub fn log(&mut self, message: &str) {
        if self.enabled {
            if let Some(file) = &mut self.file {
                _ = write!(file, "{}", message);
            }
        }
    }
}
