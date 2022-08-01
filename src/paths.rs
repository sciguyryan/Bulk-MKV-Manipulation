use crate::utils;

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Paths {
    pub mkvtoolnix: String,
    pub temp: String,
    pub ffmpeg: String,
    pub mediainfo: String,
    pub log: String,
}

lazy_static! {
    pub static ref PATHS: Paths = Paths::from_file();
}

impl Paths {
    fn from_file() -> Paths {
        use std::{env, fs};

        let path = env::current_dir();
        if path.is_err() {
            panic!("failed to get current directory");
        }

        let mut path = path.unwrap();
        path.push("paths.json");

        if !path.exists() {
            panic!("You must specify the path to the paths data file.");
        }

        let json = fs::read_to_string(path).expect("failed to open paths data file");
        let tools_result = serde_json::from_str::<Paths>(&json);
        let tools = if let Ok(p) = tools_result {
            p
        } else {
            panic!(
                "Error attempting to parse JSON data: {:?}",
                tools_result.err()
            );
        };

        if !tools.check_paths() {
            panic!("path checks failed");
        }

        tools
    }

    pub fn check_paths(&self) -> bool {
        use std::path::Path;

        let mut check: bool = true;

        if !utils::dir_exists(&self.mkvtoolnix) {
            eprintln!("Failed to locate MkvToolNIX at {}", self.mkvtoolnix);
            check = false;
        } else {
            let path = Path::new(&self.mkvtoolnix);
            let exes = vec!["mkvextract.exe", "mkvmerge.exe"];
            for exe in exes {
                let temp = path.join(exe);
                if !temp.exists() {
                    eprintln!("Failed to MkvToolNix EXE {} at {:?}", exe, temp);
                    check = false;
                }
            }
        }

        if !utils::dir_exists(&self.temp) {
            eprintln!("Failed to locate temporary folder at {}", self.temp);
            check = false;
        }

        if !utils::file_exists(&self.ffmpeg) {
            eprintln!("Failed to locate FFMPEG at {}", self.ffmpeg);
            check = false;
        }

        if !utils::file_exists(&self.mediainfo) {
            eprintln!(
                "Failed to locate MediaInfo CLI executable at {}",
                self.mediainfo
            );
            check = false;
        }

        check
    }
}
