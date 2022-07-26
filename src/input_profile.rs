use crate::{
    conversion_params::unified::UnifiedParams, file_processor::PadType,
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
