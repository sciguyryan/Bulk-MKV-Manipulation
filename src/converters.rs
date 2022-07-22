use crate::{conversion_props::AudioProperties, paths};

use std::process::Command;

pub fn convert_audio_file(file_in: &str, file_out: &str, ap: &AudioProperties) -> bool {
    // If the arguments were not valid then None will be returned.
    let args = ap.as_ffmpeg_argument_list(file_in, file_out);
    if args == None {
        return false;
    }

    //println!("{:?}", args);

    let ffmpeg = format!("{}/ffmpeg.exe", paths::FFMPEG_BASE);
    let r = Command::new(ffmpeg)
        .args(args.unwrap())
        //.current_dir(format!("{}/{}", out_path, arg_type))
        .output()
        .expect("failed to run FFMPEG extract process");

    true
}
