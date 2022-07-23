use crate::{
    conversion_props::{AudioParameters, ConversionType},
    paths,
};

use std::process::Command;

pub fn convert_audio_file(
    file_in: &str,
    file_out: &str,
    ap: &AudioParameters,
    remove_original: bool,
) -> bool {
    // If the arguments were not valid then None will be returned.
    let args = ap.as_ffmpeg_argument_list(file_in, file_out, ConversionType::Audio);
    if args == None {
        return false;
    }

    // Run the FFMPEG command.
    run_ffmpeg(&args.unwrap());

    // Do we need to remove the original media file?
    if remove_original {
        // We do not care if this succeeds or not, since the files will be removed
        // at the end of the process in any event.
        _ = std::fs::remove_file(file_in);
    }

    true
}

fn run_ffmpeg(args: &[String]) {
    let r = Command::new(paths::FFMPEG)
        .args(args)
        .output()
        .expect("failed to run FFMPEG extract process");
}
