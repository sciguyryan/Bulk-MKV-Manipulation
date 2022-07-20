mod enums;
mod media_file;
mod mkvtoolnix;
mod paths;
mod utils;

use media_file::MediaFile;

fn main() {
    if !check_paths() {
        return;
    }

    let audio_language = "ja";
    let audio_count = 1;
    let subtitle_language = "en";
    let subtitle_count = 1;
    let keep_other = false;

    let fp = "D:\\Temp\\Original\\aaaaaaa.mkv";

    let mut mf = if let Some(mi) = MediaFile::from_path(fp) {
        mi
    } else {
        panic!("Error parsing MediaInfo JSON output.");
    };

    mf.filter_tracks(
        audio_language,
        audio_count,
        subtitle_language,
        subtitle_count,
        keep_other,
    );

    mf.extract(true, true, true);

    println!("{}", mf.media.tracks.len());
}

fn check_paths() -> bool {
    use std::path::Path;

    let mut check: bool = true;

    if !utils::dir_exists(paths::FFMPEG_BASE) {
        eprintln!("Failed to locate FFMPEG at {}", paths::FFMPEG_BASE);
        check = false;
    }

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

    if !utils::file_exists(paths::MEDIAINFO) {
        eprintln!(
            "Failed to locate MediaInfo CLI executable at {}",
            paths::MEDIAINFO
        );
        check = false;
    }

    check
}
