mod conversion_params;
mod converters;
mod file_processor;
mod media_file;
mod mkvtoolnix;
mod paths;
mod utils;

use conversion_params::{
    audio::{AudioCodec, AudioParams, OpusVbrOptions, VbrOptions},
    unified::UnifiedParams,
};
use file_processor::{FileProcessor, PadType};

fn main() {
    if !check_paths() {
        return;
    }

    let in_dir = "D:\\Temp\\Input".to_string();
    let out_file_names = "D:\\Temp\\Input\\names.txt".to_string();
    let out_dir = "D:\\Temp\\Output".to_string();
    let start_from = 1;
    let pad_type = PadType::Hundred;

    let file_processor =
        match FileProcessor::new(in_dir, out_dir, out_file_names, start_from, pad_type) {
            Some(p) => p,
            None => return,
        };

    let audio_params = AudioParams {
        codec: Some(AudioCodec::Opus),
        channels: None,
        bitrate: Some(64),
        vbr: Some(VbrOptions::Opus(OpusVbrOptions::On)),
        compression_level: Some(10),
        threads: Some(12),
    };

    let params = UnifiedParams {
        audio_languages: vec!["ja".to_string()],
        audio_count: 1,
        subtitle_languages: vec!["en".to_string()],
        subtitle_count: 1,
        keep_attachments: false,
        keep_chapters: true,
        keep_other_tracks: false,
        audio_conv_params: Some(audio_params),
        video_conv_params: None,
        subtitle_conv_params: None,
        remove_original_file: false,
        remove_temp_files: true,
    };

    file_processor.process(&params);
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
