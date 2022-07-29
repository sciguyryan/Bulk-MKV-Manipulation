use crate::{
    conversion_params::unified::{TrackFilterType, UnifiedParams},
    file_processor::PadType,
    substitutions::Substitutions,
};

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct InputProfile {
    /// The path to the directory containing the input media files.
    pub input_dir: String,
    /// The path to the directory into which the processed files should be saved.
    pub output_dir: String,
    /// The path to the output names file.
    pub output_names_file_path: String,
    /// The index that the names should start from.
    pub start_from: usize,
    /// The padding that should be applied to the index.
    pub index_pad_type: PadType,
    /// Any processing parameters that should be applied to the media file.
    pub processing_params: UnifiedParams,
    /// Substitutions to be applied when sanitizing the file titles.
    pub substitutions: Substitutions,
}

impl InputProfile {
    pub fn validate_filter_params(&self) -> bool {
        let pp = &self.processing_params;
        let audio_filter = &pp.audio_tracks.filter_by;
        let subtitle_filter = &pp.subtitle_tracks.filter_by;
        let video_filter = &pp.video_tracks.filter_by;

        // Validate the audio filtering parameters.
        let audio_valid = match audio_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::TrackId => audio_filter.track_index.is_some(),
            TrackFilterType::None => true,
        };
        if !audio_valid {
            eprintln!("Failed to validate the audio filter parameters.");
        }

        // Validate the subtitle filtering parameters.
        let subtitle_valid = match subtitle_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::TrackId => subtitle_filter.track_index.is_some(),
            TrackFilterType::None => true,
        };
        if !subtitle_valid {
            eprintln!("Failed to validate the subtitle filter parameters.");
        }

        // Validate the video filtering parameters.
        let video_valid = match video_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::TrackId => video_filter.track_index.is_some(),
            TrackFilterType::None => true,
        };
        if !video_valid {
            eprintln!("Failed to validate the video filter parameters.");
        }

        audio_valid && subtitle_valid && video_valid
    }
}
