use crate::{
    conversion_props::{AudioCodec, AudioProperties},
    converters, mkvtoolnix, paths, utils,
};

use core::fmt;
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use serde_derive::Deserialize;
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

/// This will generate sequential thread-global unique IDs for instances of this struct.
static UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
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

#[derive(Clone, Default, Deserialize, PartialEq)]
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

#[derive(Deserialize)]
pub struct MediaFile {
    /// The unique sequential ID for this file.
    #[serde(skip)]
    id: usize,

    /// The path to the media file.
    #[serde(skip)]
    pub file_path: String,

    /// The data pertaining to the media file.
    pub media: MediaFileInfo,

    /// Any attachments that might be present in the media file.
    #[serde(skip)]
    pub attachments: Vec<String>,
}

impl MediaFile {
    /// Clear the list of attachments.
    fn clear_attachments(&mut self) {
        self.attachments.clear();
    }

    pub fn convert_all_audio(&mut self, props: &AudioProperties) {
        if props.codec.is_none() {
            return;
        };

        // This is the conversion codec type, converted into the
        // local codec type. These need to be segregated as they have different purposes.
        let codec = &props.codec.clone().unwrap().into();

        // Iterate through all audio tracks.
        for t in self
            .media
            .tracks
            .iter()
            .filter(|x| x.track_type == TrackType::Audio)
        {
            let file_name = t.get_out_file_name();
            let in_file_path = format!("{}\\tracks\\{}", self.get_full_temp_path(), file_name);
            let mut out_file_path = in_file_path.clone();
            //eprintln!("file_path = {}", in_file_path);

            if let Some(ext) = utils::get_file_extension(&in_file_path) {
                let out_ext = MediaFileTrack::get_extension_from_codec(codec);

                out_file_path =
                    out_file_path.replace(&format!(".{}", ext), &format!(".{}", out_ext));

                //eprintln!("out_file_path = {:?}", out_file_path);

                converters::convert_audio_file(&in_file_path, &out_file_path, props);
            }
        }
    }

    /// Create a [`MediaFile] instance from a media file path.
    /// # Arguments
    ///
    /// * `fp` - The path to the media file.
    ///
    pub fn from_path(fp: &str) -> Option<Self> {
        if !utils::file_exists(fp) {
            return None;
        }

        // Run the MediaInfo CLI process and grab the JSON output.
        let output = Command::new(paths::MEDIAINFO)
            .arg("--Output=JSON")
            .arg(fp)
            .output()
            .expect("failed to run mediainfo process");

        // Attempt to parse the JSON output.
        let json = String::from_utf8_lossy(&output.stdout).to_string();

        // We we able to successfully parse the output?
        if let Some(mut mf) = MediaFile::parse_json(&json) {
            mf.id = UNIQUE_ID.fetch_add(1, Ordering::SeqCst);

            // Set the media file path variable.
            mf.file_path = fp.to_string();

            // Do we have any attachments? If so, copy them to the main struct.
            mf.attachments = mf.media.tracks[0].extra_info.attachments.clone();

            // Set up the temporary directory structure for the file.
            mf.init_temp_directory();

            // Return the MediaFile object.
            Some(mf)
        } else {
            None
        }
    }

    /// Filter the media file tracks based on the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `audio_lang` - A list of language codes to be kept for audio files.
    /// * `audio_count` - The total number of audio tracks to be kept.
    /// * `subtitle_lang` - A list of language codes to be kept for subtitle files.
    /// * `subtitle_count` - The total number of subtitle tracks to be kept.
    /// * `keep_other` - A boolean indicating whether tracks other than video, audio and subtitle should be kept.
    ///
    pub fn filter_tracks(
        &mut self,
        audio_lang: &[&str],
        audio_count: usize,
        subtitle_lang: &[&str],
        subtitle_count: usize,
        keep_other: bool,
    ) {
        // Create a new vector to hold the tracks that we want to keep.
        let mut kept_tracks = Vec::new();

        let mut audio_kept = 0;
        let mut subs_kept = 0;

        for track in &mut self.media.tracks {
            let keep = match track.track_type {
                TrackType::Audio => {
                    audio_kept < audio_count && audio_lang.contains(&&*track.language)
                }
                // I haven't even encountered one of these before.
                TrackType::Button => keep_other,
                // This isn't a true track.
                TrackType::General => false,
                TrackType::Video => true,
                TrackType::Subtitle => {
                    subs_kept < subtitle_count && subtitle_lang.contains(&&*track.language)
                }
                TrackType::Other => keep_other,
            };

            if keep {
                // Add the track to the kept list.
                kept_tracks.push(track.clone());

                // Update the relevant counters.
                if track.track_type == TrackType::Audio {
                    audio_kept += 1;
                } else if track.track_type == TrackType::Subtitle {
                    subs_kept += 1;
                }
            }
        }

        // Assign the kept tracks back into the container object.
        self.media.tracks = kept_tracks;
    }

    fn get_full_temp_path(&self) -> String {
        let p = Path::new(paths::TEMP_BASE).join(self.id.to_string());
        p.to_string_lossy().to_string()
    }

    fn get_temp_dir_for_output_type(&self, output_type: &str) -> String {
        Path::new(paths::TEMP_BASE)
            .join(self.id.to_string())
            .join(output_type)
            .to_string_lossy()
            .to_string()
    }

    fn init_temp_directory(&self) {
        let sub_dirs = vec!["attachments", "chapters", "tracks"];

        // Create each subdirectory.
        for dir in sub_dirs {
            let p = self.get_temp_dir_for_output_type(dir);
            fs::create_dir_all(p);
        }
    }

    pub fn extract_attachments(&self) {
        // Do we have any attachments to extract?
        // The attachments will always be found on the first
        // track of the file.
        if self.attachments.is_empty() {
            return;
        }

        let mut args = Vec::new();
        for (i, attachment) in self.attachments.iter().enumerate() {
            // Note: attachments indices do not start at index 0,
            // so we have to add one to each of the IDs.
            args.push(format!("{}:{}", i + 1, attachment));
        }

        mkvtoolnix::run_mkv_extract(
            &self.file_path,
            &self.get_full_temp_path(),
            "attachments",
            &args,
        );
    }

    pub fn extract(&self, extract_tracks: bool, extract_attachments: bool, extract_chapters: bool) {
        if extract_tracks {
            self.extract_tracks();
        }

        if extract_attachments {
            self.extract_attachments();
        }

        if extract_chapters {
            self.extract_chapters();
        }
    }

    pub fn extract_chapters(&self) {
        mkvtoolnix::run_mkv_extract(
            &self.file_path,
            &self.get_full_temp_path(),
            "chapters",
            &["chapters.xml".to_string()],
        );
    }

    pub fn extract_tracks(&self) {
        let tracks = &self.media.tracks;
        if tracks.is_empty() {
            return;
        }

        let mut args = Vec::new();
        for track in tracks {
            // Note: track indices start at index 0.
            args.push(format!("{}:{}", track.id, track.get_out_file_name()));
        }

        mkvtoolnix::run_mkv_extract(&self.file_path, &self.get_full_temp_path(), "tracks", &args);
    }

    fn parse_json(json: &str) -> Option<MediaFile> {
        if let Ok(mi) = serde_json::from_str::<MediaFile>(json) {
            Some(mi)
        } else {
            None
        }
    }
}

#[derive(Deserialize)]
pub struct MediaFileInfo {
    /// A list of track that are found within the media file.
    #[serde(rename = "track")]
    pub tracks: Vec<MediaFileTrack>,
}

#[derive(Clone, Deserialize)]
pub struct MediaFileTrack {
    /// The track type field.
    ///
    /// `Note:` The [`TrackType::General`] indicates general information about the file, rather than describing a specific track.
    #[serde(rename = "@type")]
    pub track_type: TrackType,

    /// The index of the track.
    ///
    /// `Note:` [`TrackType::General`] tracks do not have an index, and so will be assigned a default value of -1.
    #[serde(rename = "StreamOrder", deserialize_with = "string_to_u32", default)]
    pub id: u32,

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

impl MediaFileTrack {
    /// Get the formatted output name for this track.
    pub fn get_out_file_name(&self) -> String {
        let ext = MediaFileTrack::get_extension_from_codec(&self.codec);

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

#[derive(Clone, Default, Deserialize)]
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
        | "A_AAC/MPEG4/LTP" | "A_AAC-2" => Codec::Aac,
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

fn string_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    match string.parse::<u32>() {
        Ok(n) => Ok(n),
        Err(_) => Err(de::Error::invalid_value(
            Unexpected::Str(&string),
            &"expected an unsigned integer",
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

fn default_track_language() -> String {
    "en".to_string()
}
