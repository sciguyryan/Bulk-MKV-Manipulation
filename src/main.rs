mod conversion_params;
mod converters;
mod file_processor;
mod input_profile;
mod logger;
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

    if args.len() == 3 {
        // Do we need to enable logging?
        if args[2].to_lowercase() == "--logging" {
            logger::set_enabled(true);
        }
    }

    logger::section("Initial Setup", false);

    if args.len() < 2 {
        logger::log(
            "No path to the conversion profile data file was specified.",
            true,
        );
        return;
    }

    // Read and parse the conversion profile data file.
    let profile_path = &args[1];
    if !utils::file_exists(profile_path) {
        logger::log(
            "The path to the conversion profile data file was invalid.",
            true,
        );
        return;
    }

    let profile_json = fs::read_to_string(profile_path).expect("failed to open profile data file");
    let profile_result = serde_json::from_str::<InputProfile>(&profile_json);
    let profile = if let Ok(p) = profile_result {
        p
    } else {
        logger::log(
            &format!(
                "An error occurred while attempting to parse the JSON data: {:?}.",
                profile_result.err()
            ),
            true,
        );
        return;
    };

    logger::log("Attempting to validate filter parameters...", false);

    // Validate the track filter parameters.
    if !profile.validate_filter_params() {
        return;
    }

    logger::log("All parameters successfully validated.", false);

    // Create the file processor instance.
    let file_processor = match FileProcessor::new(&profile) {
        Some(p) => p,
        None => return,
    };

    // Run the converter.
    file_processor.process(&profile.processing_params);
}
