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

    // Read and parse the conversion profile data file.
    assert!(
        args.len() >= 2,
        "No path to the conversion profile data file was specified."
    );
    let profile_path = &args[1];
    assert!(
        utils::file_exists(profile_path),
        "The path to the conversion profile data file was invalid."
    );

    let profile_json = fs::read_to_string(profile_path).expect("failed to open profile data file");
    let profile = serde_json::from_str::<InputProfile>(&profile_json);
    assert!(
        profile.is_ok(),
        "An error occurred while attempting to parse the JSON data: {:?}.",
        profile.err()
    );

    logger::log("Attempting to validate filter parameters...", false);

    let mut profile = profile.unwrap();

    // Validate the index processing parameters.
    if !profile.validate_index_params() {
        return;
    }

    // Initialize any regular expression filters that may be present.
    if !profile.initialize_filters() {
        return;
    }

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
