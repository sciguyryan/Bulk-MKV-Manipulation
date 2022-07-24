mod conversion_params;
mod converters;
mod file_processor;
mod input_profile;
mod media_file;
mod mkvtoolnix;
mod paths;
mod utils;

use file_processor::FileProcessor;
use input_profile::InputProfile;
use std::{env, fs};

fn main() {
    if !check_paths() {
        return;
    }

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("You must specify the path to the conversion profile data file.");
        return;
    }

    let fp = &args[1];
    if !utils::file_exists(fp) {
        eprintln!("You must specify the path to the conversion profile data file.");
        return;
    }

    // Read the contents of the conversion profile data file.
    let json = fs::read_to_string(fp).expect("failed to open profile data file");

    // Attempt to parse the conversion profile data file.
    let result = serde_json::from_str::<InputProfile>(&json);
    let profile = if let Ok(p) = result {
        p
    } else {
        println!("Error attempting to parse JSON data: {:?}", result.err());
        return;
    };

    // Create the file processor instance.
    let file_processor = match FileProcessor::new(
        profile.in_dir,
        profile.out_dir,
        profile.output_names_path,
        profile.start_from,
        profile.index_pad_type,
    ) {
        Some(p) => p,
        None => return,
    };

    // Run the converter.
    file_processor.process(&profile.processing_params);
}

fn check_paths() -> bool {
    use std::path::Path;

    let mut check: bool = true;

    if !utils::dir_exists(paths::MKVTOOLNIX_BASE) {
        eprintln!("Failed to locate MkvToolNIX at {}", paths::MKVTOOLNIX_BASE);
        check = false;
    } else {
        let path = Path::new(paths::MKVTOOLNIX_BASE);
        let exes = vec!["mkvextract.exe", "mkvmerge.exe"];
        for exe in exes {
            let temp = path.join(exe);
            if !temp.exists() {
                eprintln!("Failed to MkvToolNIX EXE {} at {:?}", exe, temp);
                check = false;
            }
        }
    }

    if !utils::dir_exists(paths::TEMP_BASE) {
        eprintln!("Failed to locate temporary folder at {}", paths::TEMP_BASE);
        check = false;
    }

    if !utils::file_exists(paths::FFMPEG) {
        eprintln!("Failed to locate FFMPEG at {}", paths::FFMPEG);
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
