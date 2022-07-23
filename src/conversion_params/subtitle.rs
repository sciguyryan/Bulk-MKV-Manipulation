use super::params::ConversionParams;

pub struct SubtitleParams {}

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
