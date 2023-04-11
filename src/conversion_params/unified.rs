use serde::{Deserialize, Deserializer};
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
    /// Parameters related to track ordering, forced tracks, etc.
    pub track_params: Option<Vec<TrackParams>>,
    /// Parameters related to the attachments.
    pub attachments: AttachmentParams,
    /// Parameters related to the chapters.
    pub chapters: ChapterParams,
    // Various other parameters that do not fit into another category.
    pub misc: MiscParams,
}

#[derive(Deserialize, Clone)]
pub struct TrackParams {
    /// The ID of the track to which these parameters should be applied.
    pub id: usize,
    /// Should the track be set as default?
    pub default: Option<bool>,
    /// Should the track be set as enabled? If unset, the track will be enabled by default.
    pub enabled: Option<bool>,
    /// Should the track be set as forced?
    pub forced: Option<bool>,
    /// Should the track be set as being for the hearing impaired?
    pub hearing_impaired: Option<bool>,
    /// Should the track be set as being for the visual impaired?
    pub visual_impaired: Option<bool>,
    /// Should the track be set being for text descriptions?
    pub text_descriptions: Option<bool>,
    /// Is the track in the original content's language?
    pub original: Option<bool>,
    /// Does the track contain commentary?
    pub commentary: Option<bool>,
    /// Should the delay be overridden for this track?
    pub delay_override: Option<i32>,
}

#[derive(Deserialize)]
pub struct AttachmentParams {
    /// Should attachments be imported from the original file?
    /// Any existing attachments will be exported and included in the final file.
    pub import_from_original: bool,
    /// The list of attachment extensions to be included from the original file.
    /// An empty list will indicate that all files should be included.
    #[serde(deserialize_with = "array_to_lowercase_string_vec")]
    pub import_original_extensions: Option<Vec<String>>,
    /// The path to a folder from which all files should be imported
    /// as attachments.
    pub import_from_folder: Option<String>,
    /// The list of attachment extensions to be included from the folder.
    /// An empty list will indicate that all files should be included.
    #[serde(deserialize_with = "array_to_lowercase_string_vec")]
    pub import_folder_extensions: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ChapterParams {
    /// Should chapters be imported from the original file?
    /// Any existing chapters will be exported and included in the final file.
    pub import_from_original: bool,
    /// Should chapters be created, if no chapters are present?
    pub create_if_not_present: bool,
    /// The interval for creating chapters, must be in the following format: HH:MM:SS.nnnnnnnnn.
    /// If `None` is specified then chapters will be created at 5 minute intervals.
    pub create_interval: Option<String>,
}

#[derive(Default, Deserialize, PartialEq, Eq, Clone)]
pub enum DeletionOptions {
    /// Delete the file using the default method.
    Delete,
    /// Delete the file by sending it to the trash, if possible.
    Trash,
    /// Do not delete the file.
    #[default]
    None,
}

#[derive(Deserialize)]
pub struct MiscParams {
    /// The method to be used when removing the original files, if specified.
    pub remove_original_file: Option<DeletionOptions>,
    /// The method to be used when removing the temporary files, if specified.
    pub remove_temp_files: Option<DeletionOptions>,
    // Should the title of the media file be set?
    pub set_file_title: bool,
    /// Should the computer be shutdown after the processing is complete?
    pub shutdown_upon_completion: bool,
    /// The path to the tags file.
    /// If the path is not specified, or is invalid, then no tags will be added.
    pub tags_path: Option<String>,
    /// The paths to any processes that should be run before or after this processing step.
    pub run: Option<ProcessRun>,
}

#[derive(Deserialize)]
pub struct UnifiedAudioParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for audio tracks.
    pub conversion: Option<AudioParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
}

#[derive(Deserialize)]
pub struct UnifiedSubtitleParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<SubtitleParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
}

#[derive(Deserialize)]
pub struct UnifiedOtherTrackParams {
    /// Should "other" be imported from the original file?
    /// These are tracks that are not of type: audio, subtitle or video.
    pub import_from_original: bool,
}

#[derive(Deserialize)]
pub struct UnifiedVideoParams {
    /// How should the tracks be filtered?
    pub filter_by: TrackFilterBy,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<VideoParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
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

#[derive(Deserialize, Clone)]
pub struct ProcessRun {
    /// The path to the process to be run before this stage is initiated.
    pub pre_mux: Option<Vec<String>>,
    /// The path to the process to be run after this stage has been completed.
    pub post_mux: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub enum ProcessRunType {
    // A command to be run prior to muxing.
    PreMux,
    // A command to be run after muxing.
    PostMux,
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

fn array_to_lowercase_string_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut vec: Vec<String> = Vec::deserialize(deserializer)?;

    for v in &mut vec {
        *v = v.to_lowercase()
    }

    Ok(Some(vec))
}
