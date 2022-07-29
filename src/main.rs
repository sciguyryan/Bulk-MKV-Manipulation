mod conversion_params;
mod converters;
mod file_processor;
mod input_profile;
mod media_file;
mod mkvtoolnix;
mod paths;
mod substitutions;
mod utils;

use file_processor::FileProcessor;
use input_profile::InputProfile;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("You must specify the path to the conversion profile data file.");
        return;
    }

    // Read and parse the conversion profile data file.
    let profile_path = &args[1];
    if !utils::file_exists(profile_path) {
        eprintln!("You must specify the path to the conversion profile data file.");
        return;
    }

    let profile_json = fs::read_to_string(profile_path).expect("failed to open profile data file");
    let profile_result = serde_json::from_str::<InputProfile>(&profile_json);
    let profile = if let Ok(p) = profile_result {
        p
    } else {
        println!(
            "Error attempting to parse JSON data: {:?}",
            profile_result.err()
        );
        return;
    };

    // Validate the track filter parameters.
    if !profile.validate_filter_params() {
        return;
    }

    // Create the file processor instance.
    let file_processor = match FileProcessor::new(&profile) {
        Some(p) => p,
        None => return,
    };

    // Run the converter.
    file_processor.process(&profile.processing_params);
}
