use crate::{conversion_params::unified::UnifiedParams, file_processor::PadType};

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct InputProfile {
    pub in_dir: String,
    pub out_dir: String,
    pub output_names_path: String,
    pub start_from: usize,
    pub index_pad_type: PadType,
    pub processing_params: UnifiedParams,
}
