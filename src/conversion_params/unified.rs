use serde_derive::{Deserialize, Serialize};

use super::{audio::AudioParams, subtitle::SubtitleParams, video::VideoParams};

#[derive(Deserialize, Serialize)]
pub struct UnifiedParams {
    /// Parameters related to the audio tracks.
    pub audio_tracks: UnifiedAudioParams,
    /// Parameters related to the subtitle tracks.
    pub subtitle_tracks: UnifiedSubtitleParams,
    /// Parameters related to the video tracks.
    pub video_tracks: UnifiedVideoParams,
    /// Parameters related to any other type of track.
    pub other_tracks: UnifiedOtherTrackParams,
    /// Parameters related to the attachments.
    pub attachments: AttachmentParams,
    /// Parameters related to the chapters.
    pub chapters: ChapterParams,
    // Various other parameters that do not fit into another category.
    pub misc_params: MiscParams,
}

#[derive(Deserialize, Serialize)]
pub struct AttachmentParams {
    /// Should attachments be included in the final file?
    /// Any existing attachments will be exported and included in the final file.
    pub include: bool,
    /// The list of file extensions to be included in the final file.
    /// An empty list will indicate that all files should be included.
    pub include_extensions: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ChapterParams {
    /// Should chapters be included in the final file?
    /// Any existing chapters will be exported and included in the final file.
    pub include: bool,
    /// Should chapters be created, if no chapters are present?
    pub create_if_not_present: bool,
    /// The interval for creating chapters, must be in the following format: HH:MM:SS.nnnnnnnnn.
    /// If `None` is specified then chapters will be created at 5 minute intervals.
    pub create_interval: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct MiscParams {
    /// Should the original media file be removed after processing?
    pub remove_original_file: bool,
    /// Should the temporary files be removed after processing?
    pub remove_temp_files: bool,
    // Should the title of the media file be set?
    pub set_file_title: bool,
}

#[derive(Deserialize, Serialize)]
pub struct UnifiedAudioParams {
    pub include_languages: Vec<String>,
    pub track_count: usize,
    pub conversion: Option<AudioParams>,
}

#[derive(Deserialize, Serialize)]
pub struct UnifiedSubtitleParams {
    pub include_languages: Vec<String>,
    pub track_count: usize,
    pub conversion: Option<SubtitleParams>,
}

#[derive(Deserialize, Serialize)]
pub struct UnifiedOtherTrackParams {
    /// Should "other" be included in the final file?
    /// These are tracks that are not of type: audio, subtitle or video.
    pub include: bool,
}

#[derive(Deserialize, Serialize)]
pub struct UnifiedVideoParams {
    pub track_count: usize,
    pub conversion: Option<VideoParams>,
}
