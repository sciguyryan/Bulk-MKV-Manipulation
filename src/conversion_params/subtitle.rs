pub struct SubtitleParams {}

impl SubtitleParams {
    #[allow(unused)]
    pub fn validate(&self) -> bool {
        true
    }

    #[allow(unused)]
    pub fn as_ffmpeg_argument_list(&self, file_in: &str, file_out: &str) -> Option<Vec<String>> {
        if !self.validate() {
            return None;
        }

        Some(vec![])
    }
}
