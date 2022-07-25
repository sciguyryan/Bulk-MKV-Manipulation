use crate::{
    conversion_params::{
        audio::{AudioCodec, AudioParams},
        subtitle::SubtitleParams,
        unified::UnifiedParams,
        video::VideoParams,
    },
    converters, mkvtoolnix, paths, utils,
};

use core::fmt;
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use serde_derive::Deserialize;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

/// This will generate sequential thread-global unique IDs for instances of this struct.
static UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

/// This will indicate whether the JSON MediaInfo output should be exported to a file.
const EXPORT_JSON: bool = false;

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

impl From<AudioCodec> for Codec {
    fn from(ac: AudioCodec) -> Self {
        match ac {
            AudioCodec::Aac => Codec::Aac,
            AudioCodec::AacLibfdk => Codec::Aac,
            AudioCodec::Ac3 => Codec::Ac3,
            AudioCodec::Flac => Codec::Flac,
            AudioCodec::Mp2 => Codec::Mp2,
            AudioCodec::Mp3Lame => Codec::Mp3,
            AudioCodec::Mp3Shine => Codec::Mp3,
            AudioCodec::Opus => Codec::Opus,
            AudioCodec::Vorbis => Codec::Opus,
            AudioCodec::WavPack => Codec::WavPack4,
        }
    }
}

#[derive(Clone, Default, Deserialize, PartialEq)]
pub enum TrackType {
    /// An audio track.
    Audio,
    /// A button track, not something that is useful here.
    Button,
    /// A general data pseudo-track.
    General,
    /// A menu track. This is how chapters are typically displayed.
    Menu,
    /// A track that does not fit into any of the other categories.
    #[default]
    Other,
    /// A subtitle track.
    #[serde(rename = "Text")]
    Subtitle,
    /// A video track.
    Video,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrackType::Audio => write!(f, "audio"),
            TrackType::Button => write!(f, "button"),
            TrackType::General => write!(f, "general"),
            TrackType::Menu => write!(f, "menu"),
            TrackType::Other => write!(f, "other"),
            TrackType::Subtitle => write!(f, "subtitle"),
            TrackType::Video => write!(f, "video"),
        }
    }
}

#[derive(Clone, Default, Deserialize, PartialEq)]
pub enum DelaySource {
    Container,
    #[default]
    None,
    Stream,
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
    pub fn convert_all_audio(&mut self, params: &AudioParams) {
        if params.codec.is_none() {
            return;
        };

        // This is the conversion codec type, converted into the
        // local codec type. These need to be segregated as they have different purposes.
        let out_codec = &params.codec.clone().unwrap().into();

        // A list of the updated track indices.
        let mut update_indices = Vec::new();

        // Iterate through all audio tracks.
        for (i, t) in self
            .media
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, x)| x.track_type == TrackType::Audio)
        {
            // Determine the output file name.
            let mut in_file_path = format!(
                "{}\\tracks\\{}",
                self.get_full_temp_path(),
                t.get_out_file_name()
            );

            // Get the new file extension and set the output
            // file extension to it.
            let in_ext = MediaFileTrack::get_extension_from_codec(&t.codec);
            let out_ext = MediaFileTrack::get_extension_from_codec(out_codec);
            let out_file_path = utils::swap_file_extension(&in_file_path, &out_ext);

            // In the case where the input and output files have the same
            // name (by having the same codec type), we need to rename
            // the original to avoid attempting to overwrite the original
            // while also trying to convert it. Needless to say, that does not work.
            if in_ext == out_ext {
                let new_file_path = in_file_path.replace(
                    &t.get_out_file_name(),
                    &format!("moved{}.{}", t.id, out_ext),
                );
                let _ = fs::rename(&in_file_path, &new_file_path);

                // Ensure that we work with the new path.
                in_file_path = new_file_path;
            }

            // Was the conversion successful? If so, add the index to the list
            // so that the codec can be updated later.
            if converters::convert_audio_file(&in_file_path, &out_file_path, params) {
                update_indices.push(i);
            }
        }

        // Update the codecs of the converted tracks.
        for index in update_indices {
            self.media.tracks[index].codec = out_codec.clone();
        }
    }

    #[allow(unused)]
    pub fn convert_all_subtitles(&mut self, params: &SubtitleParams) {
        if params.codec.is_none() {
            return;
        };

        todo!("not yet implemented");
    }

    #[allow(unused)]
    pub fn convert_all_video(&mut self, params: &VideoParams) {
        if params.codec.is_none() {
            return;
        };

        todo!("not yet implemented");
    }

    #[allow(unused)]
    pub(crate) fn dump_json(json: &str) {
        use std::{fs::File, io::Write};

        let fp = utils::join_paths_to_string(paths::TEMP_BASE, &["output.json"]);

        let mut file = File::create(fp).expect("create failed");
        Write::write_all(&mut file, json.as_bytes()).expect("write failed");
    }

    /// Extract the specified items from a MKV file.
    ///
    /// # Arguments
    ///
    /// * `extract_tracks` - Should tracks be extracted?
    /// * `extract_attachments` - Should attachments be extracted?
    /// * `extract_chapters` - Should chapters be extracted?
    ///
    pub fn extract(
        &mut self,
        extract_tracks: bool,
        extract_attachments: bool,
        extract_chapters: bool,
    ) {
        if extract_tracks {
            self.extract_tracks();
        }

        if extract_attachments {
            self.extract_attachments();
        } else {
            self.attachments.clear();
        }

        if extract_chapters {
            self.extract_chapters();
        }
    }

    /// Extract the attachments from a MKV file, if present.
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

    /// Extract the chapters from a MKV file, if present.
    pub fn extract_chapters(&self) {
        mkvtoolnix::run_mkv_extract(
            &self.file_path,
            &self.get_full_temp_path(),
            "chapters",
            &["chapters.xml".to_string()],
        );
    }

    /// Extract the tracks from a MKV file.
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

    /// Filter the media file attachments based on the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `extensions` - A list of valid attachment file extensions to be included in the output file.
    ///
    pub fn filter_attachments(&mut self, params: &UnifiedParams) {
        // If we have no attachments ot an empty filter, then we have
        // nothing to do here.
        if self.attachments.is_empty() || params.attachments.include_extensions.is_empty() {
            return;
        }

        // File extension matches should be case insensitive.
        // Clippy keeps flagging this, even though it is correct and more
        // efficient than its suggestion.
        #[allow(clippy::needless_collect)]
        let lower_exts: Vec<String> = params
            .attachments
            .include_extensions
            .iter()
            .map(|x| x.to_lowercase())
            .collect();

        // Create a new vector to hold the attachments that we want to keep.
        let mut kept = Vec::new();
        for attachment in &self.attachments {
            // Get the extension of the file.
            if let Some(ext) = utils::get_file_extension(attachment) {
                // Do we need to keep this file extension?
                if lower_exts.contains(&ext) {
                    kept.push(attachment.to_string());
                }
            }
        }

        // Assign the kept attachments back into the container object.
        self.attachments = kept;
    }

    /// Filter the media file tracks based on the specified criteria.
    pub fn filter_tracks(&mut self, params: &UnifiedParams) {
        // Create a new vector to hold the tracks that we want to keep.
        let mut kept = Vec::new();

        let mut audio_kept = 0;
        let mut subs_kept = 0;
        let mut video_kept = 0;

        let audio = &params.audio_tracks;
        let subtitle = &params.subtitle_tracks;
        let video = &params.video_tracks;

        for track in &mut self.media.tracks {
            let keep = match track.track_type {
                TrackType::Audio => {
                    audio_kept < audio.track_count
                        && audio
                            .include_languages
                            .contains(&track.language.to_string())
                }
                TrackType::Button => params.other_tracks.include,
                TrackType::General => false,
                TrackType::Menu => false,
                TrackType::Other => params.other_tracks.include,
                TrackType::Subtitle => {
                    subs_kept < subtitle.track_count
                        && subtitle
                            .include_languages
                            .contains(&track.language.to_string())
                }
                TrackType::Video => video_kept < video.track_count,
            };

            if keep {
                // Add the track to the kept list.
                kept.push(track.clone());

                // Update the relevant counters.
                match track.track_type {
                    TrackType::Audio => audio_kept += 1,
                    TrackType::Subtitle => subs_kept += 1,
                    TrackType::Video => video_kept += 1,
                    _ => {}
                }
            }
        }

        if audio_kept < audio.track_count {
            eprintln!(
                "Fewer audio tracks than required for file {}.",
                self.file_path
            );
        }

        if subs_kept < subtitle.track_count {
            eprintln!(
                "Fewer subtitle tracks than required for file {}.",
                self.file_path
            );
        }

        if video_kept < video.track_count {
            eprintln!(
                "Fewer video tracks than required for file {}.",
                self.file_path
            );
        }

        // Assign the kept tracks back into the container object.
        self.media.tracks = kept;
    }

    /// Create a [`MediaFile] instance from a media file path.
    ///
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
            .expect("failed to run MediaInfo process");

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

    fn get_full_temp_path(&self) -> String {
        utils::join_paths_to_string(paths::TEMP_BASE, &[self.id.to_string().as_str()])
    }

    /// Get the path to the temporary folder for the given output type.
    ///
    /// # Arguments
    ///
    /// * `output_type` - The name of the subdirectory representing the output type.
    ///
    fn get_temp_dir_for_output_type(&self, output_type: &str) -> String {
        utils::join_paths_to_string(
            paths::TEMP_BASE,
            &[self.id.to_string().as_str(), output_type],
        )
    }

    /// Initialize the temporary directory structure for the media file.
    fn init_temp_directory(&self) -> bool {
        let sub_dirs = vec!["attachments", "chapters", "tracks"];

        // Create each subdirectory.
        let mut result = true;
        for dir in sub_dirs {
            let p = self.get_temp_dir_for_output_type(dir);
            result &= fs::create_dir_all(p).is_ok();
        }

        result
    }

    /// Process a media file, applying any conversions and filters before remuxing the file.
    ///
    /// # Arguments
    ///
    /// * `out_path` - The path of the output media file.
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn process(&mut self, out_path: &str, params: &UnifiedParams) {
        // Filter the attachments based on the filter parameters.
        self.filter_attachments(params);

        // Filter the tracks based on the filter parameters.
        self.filter_tracks(params);

        // Extract the files.
        self.extract(true, params.attachments.include, params.chapters.include);

        // Convert the audio tracks.
        if let Some(ac) = &params.audio_tracks.conversion {
            if ac.codec.is_some() {
                self.convert_all_audio(ac);
            }
        }

        // Convert the subtitle tracks.
        if let Some(_sc) = &params.subtitle_tracks.conversion {
            todo!("not yet implemented");
        }

        // Convert the video tracks.
        if let Some(_vc) = &params.video_tracks.conversion {
            todo!("not yet implemented");
        }

        // Remux the media file.
        self.remux_file(out_path, params);

        // Delete the temporary files.
        if params.remove_temp_files {
            utils::delete_directory(&self.get_full_temp_path());
        }
    }

    /// Apply the parameters related the attachments to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    fn apply_attachment_mux_params(&self, args: &mut Vec<String>) {
        // Iterate over all of the attachments.
        for attachment in &self.attachments {
            let path = format!("./attachments/{}", attachment);

            let ext = utils::get_file_extension(&path);
            if ext.is_none() {
                continue;
            }
            let mime = utils::guess_mime_from_extension(&ext.unwrap());

            // Set the attachment name.
            args.push("--attachment-name".to_string());
            args.push(attachment.clone());

            // Set the attachment mimetype.
            args.push("--attachment-mime-type".to_string());
            args.push(mime);

            // Set the attachment file path.
            args.push("--attach-file".to_string());
            args.push(path);
        }
    }

    /// Apply the parameters related the chapters to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    fn apply_chapters_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        args.push("--chapter-language".to_string());
        args.push("en".to_string());

        // Did we export an existing chapters file?
        let chapters_fp =
            utils::join_paths_to_string(&self.get_full_temp_path(), &["chapters", "chapters.xml"]);
        if utils::file_exists(&chapters_fp) {
            // Yes, include that file.
            args.push("--chapters".to_string());
            args.push(chapters_fp);
        } else if params.chapters.create_if_not_present {
            // No, we will have to create the chapters from scratch.
            args.push("--generate-chapters-name-template".to_string());
            args.push("Chapter <NUM:2>".to_string());

            args.push("--generate-chapters".to_string());

            // By default we will create chapters at intervals of
            // 5 minutes, unless a different interval is specified.
            let mut format = "00:05:00.000000000";
            if let Some(interval) = &params.chapters.create_interval {
                if !interval.is_empty() {
                    format = interval;
                }
            }
            args.push(format!("interval:{}", format));
        }
    }

    /// Apply the parameters related the tracks to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    fn apply_track_mux_params(&self, args: &mut Vec<String>) {
        // Iterate over all of the tracks.
        for track in &self.media.tracks {
            // Do we need to specify a delay for the track?
            if track.delay != 0 {
                match track.delay_source {
                    DelaySource::Container => {
                        args.push("--sync".to_string());
                        args.push(format!("0:{}", track.delay));
                    }
                    _ => {
                        todo!("not yet implemented.");
                    }
                }
            }

            // Specify the track language.
            args.push("--language".to_string());

            // Set the track language. We set undefined for any video tracks.
            if track.track_type == TrackType::Video {
                args.push("0:und".to_string());
            } else {
                args.push(format!("0:{}", track.language));
            }

            // Set the file path.
            args.push(format!("./tracks/{}", track.get_out_file_name()));
        }
    }

    /// Remux the attachments, chapters and tracks into a single file.
    ///
    /// # Arguments
    ///
    /// * `out_path` - The path to the expected location of the output media file.
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn remux_file(&self, out_path: &str, params: &UnifiedParams) {
        use std::{fmt::Write, path::Path};

        let mut args = Vec::with_capacity(100);

        // The output file path.
        args.extend_from_slice(&["-o".to_string(), out_path.to_string()]);

        // The title of the media file.
        if params.set_file_title {
            // Get the name of the output file.
            if let Some(n) = Path::new(out_path).file_stem() {
                args.extend_from_slice(&["--title".to_string(), n.to_string_lossy().to_string()]);
            }
        }

        // Apply the track muxing parameters.
        self.apply_track_mux_params(&mut args);

        // Apply the attachment muxing parameters, is we need
        // to include those in the final file.
        if params.attachments.include {
            self.apply_attachment_mux_params(&mut args);
        }

        // Do we need to include chapters?
        if params.chapters.include {
            self.apply_chapters_mux_params(&mut args, params);
        }

        // Set the track order.
        let mut order = String::new();
        for i in 0..self.media.tracks.len() {
            let _r = write!(&mut order, "{}:0", i);

            // A comma at the end would be considered malformed.
            if i < self.media.tracks.len() - 1 {
                order.push(',');
            }
        }
        args.push("--track-order".to_string());
        args.push(order);

        // Run the MKV merge process.
        mkvtoolnix::run_mkv_merge(&self.get_full_temp_path(), &args);
    }

    /// Parse the JSON output from MediaInfo.
    ///
    /// # Arguments
    ///
    /// * `json` - The JSON string to be parsed.
    fn parse_json(json: &str) -> Option<MediaFile> {
        if EXPORT_JSON {
            MediaFile::dump_json(json);
        }

        let result = serde_json::from_str::<MediaFile>(json);
        if let Ok(mi) = result {
            Some(mi)
        } else {
            println!("Error attempting to parse JSON data: {:?}", result.err());
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

    /// The delay of the tracks, in milliseconds.
    #[serde(rename = "Delay", deserialize_with = "second_string_to_ms", default)]
    pub delay: i32,

    /// If there is a track delay, what is the delay relative too?
    #[serde(
        rename = "Delay_Source",
        deserialize_with = "string_to_delay_source_enum",
        default
    )]
    pub delay_source: DelaySource,

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
    /// Get the output name for this track.
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
        "A_VORBIS" | "A_OPUS" => Codec::Opus,
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

fn string_to_delay_source_enum<'de, D>(deserializer: D) -> Result<DelaySource, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let source = match string.as_str() {
        "Container" => DelaySource::Container,
        "Stream" => DelaySource::Stream,
        _ => DelaySource::None,
    };

    Ok(source)
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

fn second_string_to_ms<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    // The string is defined in terms of seconds.
    // We will therefore first attempt to parse this value as a f32.
    match string.parse::<f32>() {
        Ok(n) => {
            // The number must be multiplied by 1000 to give the delay in milliseconds.
            let ms = n * 1000.0;
            Ok(ms as i32)
        }
        Err(_) => Err(de::Error::invalid_value(
            Unexpected::Str(&string),
            &"expected an integer",
        )),
    }
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
