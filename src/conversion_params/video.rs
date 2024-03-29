use crate::media_file::MediaFileTrack;

use core::fmt;
use serde_derive::{Deserialize, Serialize};

use super::params_trait::ConversionParams;

#[derive(Clone, Deserialize, Serialize)]
#[allow(unused)]
pub enum VideoCodec {
    None,
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VideoCodec::None => write!(f, "none"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct VideoConvertParams {
    /// The video codec to be used for the conversion.
    pub codec: Option<VideoCodec>,
}

impl ConversionParams for VideoConvertParams {
    #[allow(unused)]
    fn validate(&self) -> bool {
        true
    }

    #[allow(unused)]
    fn as_ffmpeg_argument_list(
        &self,
        track: &MediaFileTrack,
        file_in: &str,
        file_out: &str,
    ) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        Some(vec![])
    }
}
