use core::fmt;

use super::params_trait::ConversionParams;

#[allow(unused)]
#[derive(Clone)]
pub enum SubtitleCodec {
    None,
}

impl fmt::Display for SubtitleCodec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubtitleCodec::None => write!(f, "none"),
        }
    }
}

pub struct SubtitleParams {
    /// The subtitle codec to be used for the conversion.
    pub codec: Option<SubtitleCodec>,
}

impl ConversionParams for SubtitleParams {
    /// Validate the specified codec parameters.
    #[allow(unused)]
    fn validate(&self) -> bool {
        true
    }

    #[allow(unused)]
    fn as_ffmpeg_argument_list(&self, file_in: &str, file_out: &str) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        Some(vec![])
    }
}
