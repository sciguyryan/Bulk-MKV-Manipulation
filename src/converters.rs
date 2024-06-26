use crate::{
    conversion_params::{
        audio::AudioConvertParams, params_trait::ConversionParams, subtitle::SubtitleConvertParams,
        video::VideoConvertParams,
    },
    logger,
    media_file::MediaFileTrack,
    paths,
};

use std::process::Command;

const FAIL_ERROR_CODE: i32 = 1;

/// Convert an audio file, based on the specified conversion parameters.
///
/// # Arguments
///
/// * `track` - A reference to the media file track instance.
/// * `file_in` - The path to the input file.
/// * `file_out` - The path to the output file.
/// * `params` - The parameters to be used for encoding the output file.
pub fn convert_audio_file(
    track: &MediaFileTrack,
    file_in: &str,
    file_out: &str,
    params: &AudioConvertParams,
) -> bool {
    // If the arguments were not valid then None will be returned.
    let args = params.as_ffmpeg_argument_list(track, file_in, file_out);
    if args.is_none() {
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
#[allow(unused)]
pub fn convert_subtitle_file(
    file_in: &str,
    file_out: &str,
    params: &SubtitleConvertParams,
) -> bool {
    todo!("not yet implemented");
}

/// Convert a video file, based on the specified conversion parameters.
///
/// # Arguments
///
/// * `file_in` - The path to the input file.
/// * `file_out` - The path to the output file.
/// * `params` - The parameters to be used for encoding the output file.
#[allow(unused)]
pub fn convert_video_file(file_in: &str, file_out: &str, params: &VideoConvertParams) -> bool {
    todo!("not yet implemented");
}

/// Run FFMPEG to encode the media file, with the specified arguments.
///
/// # Arguments
///
/// * `args` - A list of the command-line arguments to be passed to FFMPEG.
fn run_ffmpeg(args: &[String]) -> i32 {
    let output = Command::new(&paths::PATHS.ffmpeg).args(args).output();
    let result = match &output {
        Ok(o) => o.status.code().unwrap_or(FAIL_ERROR_CODE),
        Err(_) => FAIL_ERROR_CODE,
    };

    if result == FAIL_ERROR_CODE {
        logger::log(
            "FFMPEG was not successfully executed and yielded the following output:",
            false,
        );
        let out = output.unwrap();
        logger::log_output_lines(&String::from_utf8_lossy(&out.stderr), false);
    }

    result
}
