use crate::conversion_params::{audio::AudioParams, subtitle::SubtitleParams, video::VideoParams};

pub struct UnifiedParams {
    pub audio_languages: Vec<String>,
    pub audio_count: usize,
    pub subtitle_languages: Vec<String>,
    pub subtitle_count: usize,
    pub keep_attachments: bool,
    pub keep_chapters: bool,
    pub keep_other_tracks: bool,
    pub audio_conv_params: Option<AudioParams>,
    pub video_conv_params: Option<VideoParams>,
    pub subtitle_conv_params: Option<SubtitleParams>,
}
