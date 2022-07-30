use serde_derive::Deserialize;

use super::{audio::AudioParams, subtitle::SubtitleParams, video::VideoParams};

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct AttachmentParams {
    /// Should attachments be included in the final file?
    /// Any existing attachments will be exported and included in the final file.
    pub include: bool,
    /// The list of file extensions to be included in the final file.
    /// An empty list will indicate that all files should be included.
    pub include_extensions: Vec<String>,
}

#[derive(Deserialize)]
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

#[derive(Clone, Default, Deserialize, PartialEq)]
pub enum DeletionOptions {
    /// Delete the file.
    Delete,
    /// Send the file to the trash, if possible.
    Trash,
    /// Do nothing.
    #[default]
    None,
}

#[derive(Deserialize)]
pub struct MiscParams {
    /// Should the original media file be removed after processing?
    pub remove_original_file: Option<DeletionOptions>,
    /// Should the temporary files be removed after processing?
    pub remove_temp_files: Option<DeletionOptions>,
    // Should the title of the media file be set?
    pub set_file_title: bool,
    /// Should the computer be shutdown after the processing is complete?
    pub shutdown_upon_completion: bool,
    /// The path to the tags file.
    /// If the path is not specified, or is invalid, then no tags will be added.
    pub tags_path: Option<String>,
}

#[derive(Deserialize)]
pub struct UnifiedAudioParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for audio tracks.
    pub conversion: Option<AudioParams>,
}

#[derive(Deserialize)]
pub struct UnifiedSubtitleParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<SubtitleParams>,
}

#[derive(Deserialize)]
pub struct UnifiedOtherTrackParams {
    /// Should "other" be included in the final file?
    /// These are tracks that are not of type: audio, subtitle or video.
    pub include: bool,
}

#[derive(Deserialize)]
pub struct UnifiedVideoParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<VideoParams>,
}

#[derive(Default, Deserialize)]
pub enum TrackFilterType {
    /// Filter by language code.
    Language,
    /// Filter by track ID.
    TrackId,
    /// No filter should be applied.
    #[default]
    None,
}

#[derive(Deserialize)]
pub struct TrackFilterBy {
    /// The type of filter that should be applied to this track.
    pub filter_type: TrackFilterType,
    /// The list of language codes that should be accepted.
    /// If this is empty then all languages will be accepted.
    /// Only applicable to [`FilterType::Language`].
    pub language_codes: Option<Vec<String>>,
    /// The track indices to be selected.
    pub track_indices: Option<Vec<usize>>,
    /// The number of tracks of this type to retain, in total.
    pub total_to_retain: Option<usize>,
}
