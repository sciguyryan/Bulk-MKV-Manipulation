use core::fmt;
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use serde_derive::Deserialize;

#[derive(Default, Deserialize)]
pub enum TrackType {
    Audio,
    Button,
    General,
    Video,
    #[serde(rename = "Text")]
    Subtitle,
    #[default]
    Other,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrackType::Audio => write!(f, "audio"),
            TrackType::Button => write!(f, "button"),
            TrackType::General => write!(f, "general"),
            TrackType::Video => write!(f, "video"),
            TrackType::Subtitle => write!(f, "subtitle"),
            TrackType::Other => write!(f, "other"),
        }
    }
}

#[derive(Default)]
pub enum Codec {
    Aac,
    Ac3,
    Acm,
    AdvancedSsa,
    Alac,
    Avs,
    Dts,
    DvbSubtitle,
    FfV1,
    Flac,
    H264,
    Hdmv,
    Hevc,
    Kate,
    Ms,
    Mp1,
    Mp2,
    Mp3,
    Musepack,
    Opus,
    Pcm,
    ProRes,
    QuickTime,
    Raw,
    RealAudio,
    RealVideo,
    SubStationAlpha,
    SubTextUtf8,
    SubtitleBitmap,
    Theora,
    TheTrueAudio,
    #[default]
    Unknown,
    VobSub,
    Vp8,
    Vp9,
    WavPack4,
    WebVtt,
}

#[derive(Deserialize)]
pub struct MediaInfo {
    /// The data pertaining to the media file.
    pub media: MediaInfoMedia,
}

#[derive(Deserialize)]
pub struct MediaInfoMedia {
    /// A list of track that are found within the media file.
    #[serde(rename = "track")]
    pub tracks: Vec<MediaInfoTrack>,
}

#[derive(Deserialize)]
pub struct MediaInfoTrack {
    /// The track type field.
    ///
    /// `Note:` The [`TrackType::General`] indicates general information about the file, rather than describing a specific track.
    #[serde(rename = "@type")]
    pub track_type: TrackType,

    /// The index of the track.
    ///
    /// `Note:` [`TrackType::General`] tracks do not have an index, and so will be assigned a default value of -1.
    #[serde(
        rename = "StreamOrder",
        deserialize_with = "string_to_i32",
        default = "default_track_index"
    )]
    pub id: i32,

    /// The ID of the track's codec. This will be used to determine some additional information later.
    #[serde(rename = "CodecID", deserialize_with = "string_to_codec_enum", default)]
    pub codec: Codec,

    /// The track's language ID. If this is not defined, or is specifically set to und (undefined) then it will default to English.
    #[serde(
        rename = "Language",
        deserialize_with = "string_to_language_id",
        default = "default_track_language"
    )]
    pub language: String,

    /// Is the track selected by default?
    #[serde(rename = "Default", deserialize_with = "yes_no_to_bool", default)]
    pub default: bool,

    /// The additional track information.
    ///
    /// `Note:` This field will only contains meaningful data when the [`MediaInfoTrack::track_type`] is [`TrackType::General`].
    #[serde(rename = "extra", default)]
    pub extra_info: MediaInfoExtra,
}

impl MediaInfoTrack {
    /// Get the formatted output name for this track.
    pub fn get_out_file_name(&self) -> String {
        let ext = MediaInfoTrack::get_extension_from_codec(&self.codec);

        format!("{}_{}_{}.{}", self.track_type, self.id, self.language, ext)
    }

    /// Get the file extension associated with a specific codec ID.
    pub fn get_extension_from_codec(codec: &Codec) -> String {
        let r = match codec {
            Codec::Aac => "aac",
            Codec::Ac3 => "ac3",
            Codec::Acm => "acm",
            Codec::AdvancedSsa => "ass",
            Codec::Alac => "m4a",
            Codec::Avs => "avs",
            Codec::Dts => "dts",
            Codec::FfV1 => "ffv1",
            Codec::Flac => "flac",
            Codec::H264 => "h264",
            Codec::Hevc => "hevc",
            Codec::Ms => "m2ts",
            Codec::Mp1 => "mp1",
            Codec::Mp2 => "mp2",
            Codec::Mp3 => "mp3",
            Codec::Musepack => "mpc",
            Codec::Opus => "opus",
            Codec::Pcm => "pcm",
            Codec::ProRes => "prores",
            Codec::QuickTime => "mov",
            Codec::Raw => "raw",
            Codec::RealAudio => "ra",
            Codec::RealVideo => "rm",
            Codec::SubStationAlpha => "ssa",
            Codec::DvbSubtitle | Codec::Kate | Codec::Hdmv | Codec::SubTextUtf8 => "srt",
            Codec::SubtitleBitmap => "bmp",
            Codec::Theora => "ogg",
            Codec::TheTrueAudio => "ta1",
            Codec::Unknown => "unknown",
            Codec::VobSub => "sub",
            Codec::Vp8 => "vp8",
            Codec::Vp9 => "vp9",
            Codec::WavPack4 => "wv",
            Codec::WebVtt => "vtt",
        };

        r.to_string()
    }
}

#[derive(Default, Deserialize)]
pub struct MediaInfoExtra {
    /// A list of attachments that are found within the media file.
    #[serde(
        rename = "Attachments",
        deserialize_with = "attachment_string_to_vector",
        default
    )]
    pub attachments: Vec<String>,
}

fn attachment_string_to_vector<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let mut vec = Vec::new();
    for sub in string.split(" / ") {
        vec.push(sub.to_string());
    }

    Ok(vec)
}

fn string_to_codec_enum<'de, D>(deserializer: D) -> Result<Codec, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let codec = match string.as_str() {
        // Video codecs
        "V_MS/VFW/FOURCC" => Codec::Ms,
        "V_UNCOMPRESSED" => Codec::Raw,
        "V_MPEG4/ISO/SP" | "V_MPEG4/ISO/ASP" | "V_MPEG4/ISO/AP" | "V_MPEG4/MS/V3"
        | "V_MPEG4/ISO/AVC" => Codec::H264,
        "V_MPEGH/ISO/HEVC" => Codec::Hevc,
        "V_MPEG1" => Codec::Mp1,
        "V_MPEG2" => Codec::Mp2,
        "V_AVS2" => Codec::Avs,
        "V_REAL/RV10" | "V_REAL/RV20" | "V_REAL/RV30" | "V_REAL/RV40" => Codec::RealVideo,
        "V_QUICKTIME" => Codec::QuickTime,
        "V_THEORA" => Codec::Theora,
        "V_PRORES" => Codec::ProRes,
        "V_VP8" => Codec::Vp8,
        "V_VP9" => Codec::Vp9,
        "V_FFV1" => Codec::FfV1,

        // Audio codecs.
        "A_MPEG/L3" => Codec::Mp3,
        "A_MPEG/L2" | "A_MPEG/L1" => Codec::Mp2,
        "A_PCM/INT/BIG" | "A_PCM/INT/LIT" | "A_PCM/FLOAT/IEEE" => Codec::Pcm,
        "A_MPC" => Codec::Musepack,
        "A_AC3" | "A_AC3/BSID9" | "A_AC3/BSID10" => Codec::Ac3,
        "A_ALAC" => Codec::Alac,
        "A_DTS" | "A_DTS/EXPRESS" | "A_DTS/LOSSLESS" => Codec::Dts,
        "A_VORBIS" => Codec::Opus,
        "A_FLAC" => Codec::Flac,
        "A_REAL/14_4" | "A_REAL/28_8" | "A_REAL/COOK" | "A_REAL/SIPR" | "A_REAL/RALF"
        | "A_REAL/ATRC" => Codec::RealAudio,
        "A_MS/ACM" => Codec::Acm,
        "A_AAC/MPEG2/MAIN" | "A_AAC/MPEG2/LC" | "A_AAC/MPEG2/LC/SBR" | "A_AAC/MPEG2/SSR"
        | "A_AAC/MPEG4/MAIN" | "A_AAC/MPEG4/LC" | "A_AAC/MPEG4/LC/SBR" | "A_AAC/MPEG4/SSR"
        | "A_AAC/MPEG4/LTP" => Codec::Aac,
        "A_QUICKTIME" | "A_QUICKTIME/QDMC" | "A_QUICKTIME/QDM2" => Codec::QuickTime,
        "A_TTA1" => Codec::TheTrueAudio,
        "A_WAVPACK4" => Codec::WavPack4,

        // Subtitle codecs.
        "S_TEXT/UTF8" => Codec::SubTextUtf8,
        "S_TEXT/SSA" => Codec::SubStationAlpha,
        "S_TEXT/ASS" => Codec::AdvancedSsa,
        "S_TEXT/WEBVTT" => Codec::WebVtt,
        "S_IMAGE/BMP" => Codec::SubtitleBitmap,
        "S_DVBSUB" => Codec::DvbSubtitle,
        "S_VOBSUB" => Codec::VobSub,
        "S_HDMV/PGS" | "S_HDMV/TEXTST" => Codec::Hdmv,
        "S_KATE" => Codec::Kate,

        // Other codecs.
        _ => {
            println!("Unexpected codec ID when parsing MKV file: {}", string);
            Codec::default()
        }
    };

    Ok(codec)
}

fn string_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    match string.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(de::Error::invalid_value(
            Unexpected::Str(&string),
            &"expected a signed integer",
        )),
    }
}

fn string_to_language_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    // We specifically want to map tracks with an undefined language (und)
    // to "en" (English) to avoid them being missed.
    if string == "und" {
        return Ok(default_track_language());
    }

    Ok(string)
}

fn yes_no_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    match string.to_lowercase().as_str() {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(de::Error::invalid_value(
            Unexpected::Str(&string),
            &"expected yes or no",
        )),
    }
}

fn default_track_index() -> i32 {
    -1
}

fn default_track_language() -> String {
    "en".to_string()
}
