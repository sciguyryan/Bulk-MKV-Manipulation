use crate::{
    conversion_params::{
        audio::AudioParams, params_trait::ConversionParams, subtitle::SubtitleParams,
        video::VideoParams,
    },
    logger, paths,
};

use std::process::Command;

/// Convert an audio file, based on the specified conversion parameters.
///
/// # Arguments
///
/// * `file_in` - The path to the input file.
/// * `file_out` - The path to the output file.
/// * `params` - The parameters to be used for encoding the output file.
///
pub fn convert_audio_file(file_in: &str, file_out: &str, params: &AudioParams) -> bool {
    // If the arguments were not valid then None will be returned.
    let args = params.as_ffmpeg_argument_list(file_in, file_out);
    if args == None {
        return false;
    }

    // Run FFMPEG with the specified parameters.
    run_ffmpeg(&args.unwrap()) == 0
}

/// Convert a subtitle file, based on the specified conversion parameters.
///
/// # Arguments
///
/// * `file_in` - The path to the input file.
/// * `file_out` - The path to the output file.
/// * `params` - The parameters to be used for encoding the output file.
///
#[allow(unused)]
pub fn convert_subtitle_file(file_in: &str, file_out: &str, params: &SubtitleParams) -> bool {
    todo!("not yet implemented");
}

/// Convert a video file, based on the specified conversion parameters.
///
/// # Arguments
///
/// * `file_in` - The path to the input file.
/// * `file_out` - The path to the output file.
/// * `params` - The parameters to be used for encoding the output file.
///
#[allow(unused)]
pub fn convert_video_file(
    file_in: &str,
    file_out: &str,
    params: &VideoParams,
    remove_original: bool,
) -> bool {
    todo!("not yet implemented");
}

/// Run FMMPEG to encode the media file, with the specified arguments.
///
/// # Arguments
///
/// * `args` - A list of the command-line arguments to be passed to FFMPEG.
fn run_ffmpeg(args: &[String]) -> i32 {
    let r = Command::new(&paths::PATHS.ffmpeg).args(args).output();

    if let Ok(exit) = r {
        if let Some(code) = exit.status.code() {
            code
        } else {
            logger::log(format!("{:?}", exit.stderr), false);
            1
        }
    } else {
        1
    }
}
