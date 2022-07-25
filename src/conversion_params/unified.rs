use serde_derive::{Deserialize, Serialize};

use super::{audio::AudioParams, subtitle::SubtitleParams, video::VideoParams};

#[derive(Deserialize, Serialize)]
pub struct UnifiedParams {
    pub audio_languages: Vec<String>,
    pub audio_count: usize,
    pub subtitle_languages: Vec<String>,
    pub subtitle_count: usize,
    pub attachments: AttachmentParams,
    pub chapters: ChapterParams,
    pub other_tracks: OtherTrackParams,
    pub audio_conversion: Option<AudioParams>,
    pub video_conversion: Option<VideoParams>,
    pub subtitle_conversion: Option<SubtitleParams>,
    pub remove_original_file: bool,
    pub remove_temp_files: bool,
}

#[derive(Deserialize, Serialize)]
pub struct AttachmentParams {
    /// Should attachments be included in the final file?
    /// Any existing attachments will be exported and included in the final file.
    pub include: bool,
}

#[derive(Deserialize, Serialize)]
pub struct ChapterParams {
    /// Should chapters be included in the final file?
    /// Any existing chapters will be exported and included in the final file.
    pub include: bool,
    /// Should chapters be created, if no chapters are present?
    pub create_if_not_present: bool,
    /// The interval for creating chapters, must be in the following format: HH:MM:SS.nnnnnnnnn.
    ///
    /// If None is specified then chapters will be created at 5 minute intervals.
    pub create_interval: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct OtherTrackParams {
    /// Should "other" be included in the final file?
    /// These are tracks that are not of type: audio, subtitle or video.
    pub include: bool,
}
