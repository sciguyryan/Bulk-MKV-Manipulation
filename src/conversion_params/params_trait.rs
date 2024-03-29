use crate::media_file::MediaFileTrack;

pub trait ConversionParams {
    fn validate(&self) -> bool;
    fn as_ffmpeg_argument_list(
        &self,
        track: &MediaFileTrack,
        file_in: &str,
        file_out: &str,
    ) -> Option<Vec<String>>;
}
