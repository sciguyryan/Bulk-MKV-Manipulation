mod conversion_props;
mod converters;
mod media_file;
mod mkvtoolnix;
mod paths;
mod utils;

use conversion_props::{AudioCodec, AudioParameters, OpusVbrOptions, VbrOptions};
use media_file::MediaFile;

fn main() {
    if !check_paths() {
        return;
    }

    // Clear the temporary files.
    utils::delete_directory(paths::TEMP_BASE);

    let threads = 8;
    let audio_language = "ja";
    let audio_count = 1;
    let subtitle_language = "en";
    let subtitle_count = 1;
    let keep_other = false;

    let fp = "D:\\Temp\\Original\\aaaaaaa.mkv";
    let out_path = "E:\\muxed.mkv";

    let mut mf = if let Some(mi) = MediaFile::from_path(fp) {
        mi
    } else {
        panic!("Error parsing MediaInfo JSON output.");
    };

    mf.filter_tracks(
        &[audio_language],
        audio_count,
        &[subtitle_language],
        subtitle_count,
        keep_other,
    );

    mf.extract(true, true, true);

    let audio_props = AudioParameters {
        codec: Some(AudioCodec::Opus),
        channels: None,
        bitrate: Some(64),
        vbr: Some(VbrOptions::Opus(OpusVbrOptions::On)),
        compression_level: Some(10),
        threads: Some(threads),
    };

    // Convert the audio tracks.
    mf.convert_all_audio(&audio_props);

    // Mux the media file.
    mf.remux_file(out_path);
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
