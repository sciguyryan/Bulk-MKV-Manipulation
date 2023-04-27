use crate::{
    conversion_params::unified::{TrackFilterType, UnifiedParams},
    file_processor::PadType,
    logger,
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
    pub start_from: Option<usize>,
    /// The padding that should be applied to the index.
    pub index_pad_type: Option<PadType>,
    /// Any processing parameters that should be applied to the media file.
    pub processing_params: UnifiedParams,
    /// Substitutions to be applied when sanitizing the file titles.
    pub substitutions: Substitutions,
}

impl InputProfile {
    pub fn initialize_filters(&mut self) -> bool {
        self.processing_params
            .subtitle_tracks
            .filter_by
            .initialize_regex()
            && self
                .processing_params
                .subtitle_tracks
                .filter_by
                .initialize_regex()
            && self
                .processing_params
                .video_tracks
                .filter_by
                .initialize_regex()
    }

    pub fn validate_index_params(&self) -> bool {
        self.start_from.is_some() && self.index_pad_type.is_some()
            || self.start_from.is_none() && self.index_pad_type.is_none()
    }

    pub fn validate_filter_params(&self) -> bool {
        let pp = &self.processing_params;
        let audio_filter = &pp.audio_tracks.filter_by;
        let subtitle_filter = &pp.subtitle_tracks.filter_by;
        let video_filter = &pp.video_tracks.filter_by;

        // Validate the audio filtering parameters.
        let audio_valid = match audio_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::Title => audio_filter.track_title_filter.is_some(),
            TrackFilterType::TrackId => audio_filter.track_indices.is_some(),
            TrackFilterType::None => true,
        };
        if !audio_valid {
            logger::log("Failed to validate the audio filter parameters", true);
        }

        // Validate the subtitle filtering parameters.
        let subtitle_valid = match subtitle_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::Title => subtitle_filter.track_title_filter.is_some(),
            TrackFilterType::TrackId => subtitle_filter.track_indices.is_some(),
            TrackFilterType::None => true,
        };
        if !subtitle_valid {
            logger::log("Failed to validate the subtitle filter parameters", true);
        }

        // Validate the video filtering parameters.
        let video_valid = match video_filter.filter_type {
            TrackFilterType::Language => true,
            TrackFilterType::Title => video_filter.track_title_filter.is_some(),
            TrackFilterType::TrackId => video_filter.track_indices.is_some(),
            TrackFilterType::None => true,
        };
        if !video_valid {
            logger::log("Failed to validate the subtitle filter parameters", true);
        }

        audio_valid && subtitle_valid && video_valid
    }
}
