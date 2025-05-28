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

    if args.len() < 2 {
        eprintln!("Usage: {} <profile_path> [--logging]", args[0]);
        return;
    }

    if args
        .get(2)
        .map(|a| a.eq_ignore_ascii_case("--logging"))
        .unwrap_or(false)
    {
        logger::set_enabled(true);
    }

    logger::section("Initial Setup", false);

    let profile_path = &args[1];
    if !utils::file_exists(profile_path) {
        eprintln!("The path to the conversion profile data file was invalid.");
        return;
    }

    let profile_json = fs::read_to_string(profile_path).expect("Failed to open profile data file");
    let mut profile: InputProfile = serde_json::from_str(&profile_json)
        .expect("An error occurred while attempting to parse the JSON data.");

    logger::log("Attempting to validate filter parameters...", false);

    if !profile.validate_index_params() {
        return;
    }

    if !profile.initialize_filters() {
        return;
    }

    if !profile.validate_filter_params() {
        return;
    }

    logger::log("All parameters successfully validated.", false);

    let file_processor = match FileProcessor::new(&profile) {
        Some(p) => p,
        None => return,
    };

    file_processor.process(&profile.processing_params);
}
