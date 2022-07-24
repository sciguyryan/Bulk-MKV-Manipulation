use core::fmt;

use super::params_trait::ConversionParams;

#[allow(unused)]
#[derive(Clone)]
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

pub struct VideoParams {
    /// The subtitle codec to be used for the conversion.
    pub codec: Option<VideoCodec>,
}

impl ConversionParams for VideoParams {
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
