use crate::{
    conversion_params::{
        audio::AudioParams, params::ConversionParams, subtitle::SubtitleParams, video::VideoParams,
    },
    paths,
};

use std::process::Command;

pub fn convert_audio_file(
    file_in: &str,
    file_out: &str,
    params: &AudioParams,
    remove_original: bool,
) -> bool {
    // If the arguments were not valid then None will be returned.
    let args = params.as_ffmpeg_argument_list(file_in, file_out);
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

#[allow(unused)]
pub fn convert_subtitle_file(
    file_in: &str,
    file_out: &str,
    params: &SubtitleParams,
    remove_original: bool,
) -> bool {
    todo!("not yet implemented");
}

#[allow(unused)]
pub fn convert_video_file(
    file_in: &str,
    file_out: &str,
    params: &VideoParams,
    remove_original: bool,
) -> bool {
    todo!("not yet implemented");
}

fn run_ffmpeg(args: &[String]) {
    let _r = Command::new(paths::FFMPEG)
        .args(args)
        .output()
        .expect("failed to run FFMPEG extract process");
}
