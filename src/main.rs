mod media_info;
mod paths;
mod utils;

use std::{fs::File, io::Write, process::Command};

use media_info::MediaInfo;

fn main() {
    if !check_paths() {
        return;
    }

    let fp = "D:\\Temp\\Original\\aaaaaaa.mkv";
    let json = get_mediainfo_json(fp);
    //println!("{}", a);temp_json

    // Temporarily write the JSON data to a file.
    /*let temp_json = "E:\\Temp\\output.json";
    let mut file = File::create(temp_json).expect("failed to create temp file");
    write!(file, "{}", json);*/

    let info = if let Some(mi) = parse_json(&json) {
        mi
    } else {
        panic!("Error parsing MediaInfo JSON output.");
    };

    println!("{}", info.media.tracks.len());
}

fn check_paths() -> bool {
    let mut check: bool = true;

    if !utils::dir_exists(paths::FFMPEG_BASE) {
        eprintln!("Failed to locate FFMPEG at {}", paths::FFMPEG_BASE);
        check = false;
    }

    if !utils::dir_exists(paths::MKVTOOLNIX_BASE) {
        eprintln!("Failed to locate MkvToolNIX at {}", paths::MKVTOOLNIX_BASE);
        check = false;
    }

    if !utils::dir_exists(paths::TEMP_BASE) {
        eprintln!("Failed to locate temporary folder at {}", paths::TEMP_BASE);
        check = false;
    }

    if !utils::file_exists(paths::MEDIAINFO) {
        eprintln!(
            "Failed to locate MediaInfo CLI executable at {}",
            paths::MEDIAINFO
        );
        check = false;
    }

    check
}

fn get_mediainfo_json(fp: &str) -> String {
    let output = Command::new(paths::MEDIAINFO)
        .arg("--Output=JSON")
        .arg(fp)
        .output()
        .expect("failed to run mediainfo process");

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn parse_json(json: &str) -> Option<MediaInfo> {
    if let Ok(mi) = serde_json::from_str::<MediaInfo>(json) {
        Some(mi)
    } else {
        None
    }
}
