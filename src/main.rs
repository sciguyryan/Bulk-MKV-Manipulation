mod conversion_params;
mod converters;
mod media_file;
mod media_process_params;
mod mkvtoolnix;
mod paths;
mod utils;

use conversion_params::audio::{AudioCodec, AudioParams, OpusVbrOptions, VbrOptions};
use media_file::MediaFile;
use media_process_params::MediaProcessParams;

fn main() {
    if !check_paths() {
        return;
    }

    // Clear the temporary files.
    utils::delete_directory(paths::TEMP_BASE);

    let fp = "D:\\Temp\\Original\\aaaaaaa.mkv";
    let out_path = "E:\\muxed.mkv";

    let mut mf = if let Some(mi) = MediaFile::from_path(fp) {
        mi
    } else {
        panic!("Error parsing MediaInfo JSON output.");
    };

    let audio_params = AudioParams {
        codec: Some(AudioCodec::Opus),
        channels: None,
        bitrate: Some(64),
        vbr: Some(VbrOptions::Opus(OpusVbrOptions::On)),
        compression_level: Some(10),
        threads: Some(8),
    };

    let params = MediaProcessParams {
        audio_languages: vec!["ja".to_string()],
        audio_count: 1,
        subtitle_languages: vec!["en".to_string()],
        subtitle_count: 1,
        keep_attachments: true,
        keep_chapters: true,
        keep_other_tracks: false,
        audio_conv_params: Some(audio_params),
        video_conv_params: None,
        subtitle_conv_params: None,
    };

    mf.process(&params, out_path);
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
