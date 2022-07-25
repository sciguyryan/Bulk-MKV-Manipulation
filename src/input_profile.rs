use crate::{conversion_params::unified::UnifiedParams, file_processor::PadType};

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct InputProfile {
    pub input_dir: String,
    pub output_dir: String,
    pub output_names_file_path: String,
    pub start_from: usize,
    pub index_pad_type: PadType,
    pub processing_params: UnifiedParams,
}
