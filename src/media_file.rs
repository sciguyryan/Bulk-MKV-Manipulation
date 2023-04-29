use crate::{
    conversion_params::{
        audio::{AudioCodec, AudioParams},
        params_trait::ConversionParams,
        subtitle::SubtitleParams,
        unified::{PredicateFilterMatch, ProcessRun, TrackPredicate, UnifiedParams},
        video::VideoParams,
    },
    converters, logger, mkvtoolnix, paths, utils,
};

use core::fmt;
use serde::de::{self, Deserialize, Deserializer, Unexpected};
use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use walkdir::{DirEntry, Error, WalkDir};

/// This will generate sequential thread-global unique IDs for instances of this struct.
static UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

/// This will indicate whether the JSON MediaInfo output should be exported to a file.
const EXPORT_JSON: bool = false;

/// This will indicate whether to output the command line parameters used.
const DEBUG_PARAMS: bool = false;

#[derive(Clone, Debug, Default)]
pub enum Codec {
    Aac,
    Ac3,
    Acm,
    AdvancedSsa,
    Alac,
    Avs,
    Dts,
    DvbSubtitle,
    Eac3,
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

#[derive(Debug, PartialEq, Eq)]
pub enum RunCommandType {
    PreMux,
    PostMux,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub enum DelaySource {
    Container,
    #[default]
    None,
    Stream,
}

#[derive(Clone, Default, Deserialize, Eq, Hash, PartialEq)]
pub enum TrackType {
    /// An audio track.
    Audio,
    /// A button track.
    Button,
    /// A general data pseudo-track.
    General,
    /// A menu track. This is how chapters are typically displayed with MediaInfo.
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

#[derive(Deserialize)]
pub struct MediaFile {
    /// The unique sequential ID for this file.
    #[serde(skip)]
    id: usize,

    /// The input path to the media file.
    #[serde(skip)]
    pub file_path: String,

    /// The output for the modified media file.
    #[serde(skip)]
    pub output_path: String,

    /// The data pertaining to the media file.
    pub media: MediaFileInfo,

    /// Any attachments that might be present in the media file.
    #[serde(skip)]
    pub attachments: Vec<String>,

    /// A list of the track types and how many of each have been kept.
    #[serde(skip)]
    track_type_counter: HashMap<TrackType, usize>,
}

impl MediaFile {
    /// Apply default track languages.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters.
    pub fn apply_track_language_defaults(&mut self, params: &UnifiedParams) {
        let mut defs = HashMap::new();

        if let Some(da) = &params.audio_tracks.default_language {
            defs.insert(TrackType::Audio, da);
        }

        if let Some(ds) = &params.subtitle_tracks.default_language {
            defs.insert(TrackType::Subtitle, ds);
        }

        if let Some(dv) = &params.video_tracks.default_language {
            defs.insert(TrackType::Video, dv);
        }

        for (track_type, default_lang) in defs {
            // Iterate through all of the tracks of the specific type,
            // where the track language is "und" (undefined).
            for track in self
                .media
                .tracks
                .iter_mut()
                .filter(|t| t.track_type == track_type && t.language == "und")
            {
                track.language = default_lang.clone();
            }
        }
    }

    /// Convert each audio track found within the media file.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters to be applied to the tracks.
    pub fn convert_all_audio(&mut self, params: &AudioParams) -> bool {
        if params.codec.is_none() {
            return true;
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
            logger::log_inline(
                format!("Converting audio track {} to '{out_codec:?}'...", t.id),
                false,
            );

            // Determine the output file name.
            let mut in_file_path = t.get_input_file_path();
            let out_file_path = t.get_output_file_path(out_codec);

            let mut success = true;
            if in_file_path == out_file_path {
                // In the case where the input and output files have the same
                //   name (by having the same codec type), we need to rename
                //   the original to avoid attempting to overwrite the original
                //   while also trying to convert it. Needless to say, that does not work.
                let out_ext = MediaFileTrack::get_extension_from_codec(out_codec);
                let new_file_path = in_file_path
                    .replace(&t.get_out_file_name(), &format!("moved{}.{out_ext}", t.id));

                if fs::rename(&in_file_path, &new_file_path).is_err() {
                    logger::log(" unable to move input file, unable to encode .", false);
                    success = false;
                } else {
                    in_file_path = new_file_path;
                }
            }

            // Was the prior step successful before attempting to encode the track?
            if success {
                success = converters::convert_audio_file(t, &in_file_path, &out_file_path, params);
            }

            // Was the conversion successful? If so, add the index to the list
            // so that the codec can be updated later.
            if success {
                update_indices.push(i);
                logger::log(" conversion successful.", false);
            } else {
                logger::log(" conversion failed.", false);
            }

            // Output the FFmpeg parameters, if the debug flag is set.
            if DEBUG_PARAMS {
                let args = params
                    .as_ffmpeg_argument_list(t, &in_file_path, &out_file_path)
                    .unwrap();
                logger::log(
                    format!(
                        "ffmpeg command line: \"{}\" {}",
                        paths::PATHS.ffmpeg,
                        &args.join(" ")
                    ),
                    false,
                );
            }

            if !success {
                return false;
            }
        }

        // Update the codecs of the converted tracks.
        for index in update_indices {
            self.media.tracks[index].codec = out_codec.clone();
        }

        true
    }

    /// Convert each video track found within the media file.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters to be applied to the subtitle tracks.
    #[allow(unused)]
    pub fn convert_all_subtitles(&mut self, params: &SubtitleParams) {
        if params.codec.is_none() {
            return;
        };

        todo!("not yet implemented");
    }

    /// Convert each video tracks found within the media file.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters to be applied to the video tracks.
    #[allow(unused)]
    pub fn convert_all_video(&mut self, params: &VideoParams) {
        if params.codec.is_none() {
            return;
        };

        todo!("not yet implemented");
    }

    /// Dump the MediaInfo JSON output.
    ///
    /// # Arguments
    ///
    /// * `json`- The JSON string to be written to the file.
    #[allow(unused)]
    pub(crate) fn dump_json(json: &str) {
        use std::{fs::File, io::Write};

        let fp = utils::join_path_segments(&paths::PATHS.temp, &["output.json"]);
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
    pub fn extract(
        &mut self,
        extract_tracks: bool,
        extract_attachments: bool,
        extract_chapters: bool,
    ) -> bool {
        let mut success = true;
        if extract_tracks {
            success = self.extract_tracks();
        }

        if success && extract_attachments {
            success = self.extract_attachments();
        } else {
            self.attachments.clear();
        }

        if success && extract_chapters {
            success = self.extract_chapters();
        }

        success
    }

    /// Extract the attachments from a MKV file, if present.
    pub fn extract_attachments(&self) -> bool {
        // Do we have any attachments to extract?
        // The attachments will always be found on the first
        // track of the file.
        if self.attachments.is_empty() {
            logger::log("No attachments to extract.", false);
            return true;
        }

        logger::log_inline("Extracting attachments...", false);

        // Note: attachments indices do not start at index 0,
        // so we have to add one to each of the IDs.
        let args: Vec<String> = self
            .attachments
            .iter()
            .enumerate()
            .map(|(i, a)| format!("{}:{a}", i + 1))
            .collect();

        let r = match mkvtoolnix::run_extract(
            &self.file_path,
            &self.get_temp_path(),
            "attachments",
            &args,
        ) {
            0 | 1 => {
                logger::log(" extraction complete.", false);
                true
            }
            2 => {
                logger::log(" extraction failed.", false);
                false
            }
            _ => true,
        };

        // Output the mkvextract parameters, if the debug flag is set.
        if DEBUG_PARAMS {
            logger::log(
                format!(
                    "[info] mkvextract command line: \"{}\" {}",
                    mkvtoolnix::get_exe("mkvextract"),
                    &args.join(" ")
                ),
                false,
            );
        }

        r
    }

    /// Extract the chapters from a MKV file, if present.
    pub fn extract_chapters(&self) -> bool {
        logger::log_inline("Extracting chapters...", false);

        let r = match mkvtoolnix::run_extract(
            &self.file_path,
            &self.get_temp_path(),
            "chapters",
            &["chapters.xml".to_string()],
        ) {
            0 | 1 => {
                logger::log(" extraction complete.", false);
                true
            }
            2 => {
                logger::log(" extraction failed.", false);
                false
            }
            _ => true,
        };

        // Output the mkvextract parameters, if the debug flag is set.
        if DEBUG_PARAMS {
            logger::log(
                format!(
                    "mkvextract command line: \"{}\" chapters.xml",
                    mkvtoolnix::get_exe("mkvextract")
                ),
                false,
            );
        }

        r
    }

    /// Extract the tracks from a MKV file.
    pub fn extract_tracks(&self) -> bool {
        let tracks = &self.media.tracks;
        if tracks.is_empty() {
            logger::log("No tracks to extract.", false);
            return true;
        }

        logger::log_inline("Extracting tracks...", false);

        // Note: track indices start at index 0.
        let args: Vec<String> = tracks
            .iter()
            .map(|track| format!("{}:{}", track.id, track.get_out_file_name()))
            .collect();

        let r = match mkvtoolnix::run_extract(
            &self.file_path,
            &self.get_temp_path(),
            "tracks",
            &args,
        ) {
            0 | 1 => {
                logger::log(" extraction complete.", false);
                true
            }
            2 => {
                logger::log(" extraction failed.", false);
                false
            }
            _ => true,
        };

        // Output the mkvextract parameters, if the debug flag is set.
        if DEBUG_PARAMS {
            logger::log(
                format!(
                    "mkvextract command line: \"{}\" {}",
                    mkvtoolnix::get_exe("mkvextract"),
                    &args.join(" ")
                ),
                false,
            );
        }

        r
    }

    /// Filter the attachments from the original input file based on the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn filter_internal_attachments(&mut self, params: &UnifiedParams) {
        // If we have no attachments we can exit early.
        if self.attachments.is_empty() {
            logger::log("No attachments in original file.", false);
            return;
        }

        // If the attachment filter list is empty, we will allow all attachments
        // to be imported from the original file.
        let import_ext = match &params.attachments.import_original_extensions {
            Some(exts) => {
                if exts.is_empty() {
                    return;
                } else {
                    exts
                }
            }
            None => return,
        };

        self.attachments = self
            .attachments
            .iter()
            .filter_map(|path| {
                let ext = utils::get_file_extension(path)?;
                if import_ext.contains(&ext) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        // Assign the kept attachments back into the container object.
        logger::log(
            format!(
                "{} internal attachments kept after filtering.",
                self.attachments.len()
            ),
            false,
        );
    }

    /// Filter the media file tracks based on the specified criteria.
    ///
    /// # Arguments
    ///
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn filter_tracks(&mut self, params: &UnifiedParams) -> bool {
        // Create a new vector to hold the tracks that we want to keep.
        let mut kept = Vec::new();

        let audio = &params.audio_tracks;
        let subtitle = &params.subtitle_tracks;
        let video = &params.video_tracks;

        for (i, track) in &mut self.media.tracks.iter().enumerate() {
            let keep = self.should_keep_track(&track.track_type, i, params);

            // If we do not need to keep this track, then
            // skip to the next track.
            if !keep {
                continue;
            }

            // Add the track to the kept list.
            kept.push(track.clone());

            // Update the relevant counters.
            *self
                .track_type_counter
                .entry(track.track_type.clone())
                .or_default() += 1;
        }

        let mut success = true;
        for target_type in [TrackType::Audio, TrackType::Subtitle, TrackType::Video] {
            let target = match target_type {
                TrackType::Audio => audio.total_to_retain,
                TrackType::Subtitle => subtitle.total_to_retain,
                TrackType::Video => video.total_to_retain,
                _ => None,
            };

            if target.is_none() {
                continue;
            }

            if self
                .track_type_counter
                .get(&target_type)
                .cloned()
                .unwrap_or_default()
                != target.unwrap()
            {
                logger::log(
                    format!(
                        "Fewer tracks of type {target_type} than required for file {}.",
                        self.file_path
                    ),
                    false,
                );
                success = false;
            }
        }

        logger::log(
            format!("{} tracks kept after filtering.", kept.len()),
            false,
        );

        // Assign the kept tracks back into the container object.
        self.media.tracks = kept;

        success
    }

    /// Check whether a given track should be kept in the final file.
    ///
    /// # Arguments
    ///
    /// * `track_type` - The type of track.
    /// * `index` - The index of the track.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn should_keep_track(
        &self,
        track_type: &TrackType,
        index: usize,
        params: &UnifiedParams,
    ) -> bool {
        // We can handle certain types of tracks by type, rather than by
        // examining them in a more detailed way.
        if matches!(track_type, TrackType::General | TrackType::Menu) {
            // These tracks will never be kept.
            return false;
        } else if matches!(track_type, TrackType::Button | TrackType::Other) {
            // These tracks will only be kept is the relevant flag is set.
            return params.other_tracks.import_from_original;
        }

        // The panic should never happen since the cases are all dealt with above.
        let predicate = match track_type {
            TrackType::Audio => &params.audio_tracks.predicate,
            TrackType::Subtitle => &params.subtitle_tracks.predicate,
            TrackType::Video => &params.video_tracks.predicate,
            _ => panic!(),
        };

        // The panic should never happen since the cases are all dealt with above.
        let tracks_to_retail = match track_type {
            TrackType::Audio => &params.audio_tracks.total_to_retain,
            TrackType::Subtitle => &params.subtitle_tracks.total_to_retain,
            TrackType::Video => &params.video_tracks.total_to_retain,
            _ => panic!(),
        };

        // Is a track limiter in place, and have we reached the target number of tracks?
        if let Some(count) = tracks_to_retail {
            if let Some(c) = self.track_type_counter.get(track_type) {
                if c >= count {
                    return false;
                }
            }
        }

        // Note: that the filters are validated so the unwraps are safe here.
        match &predicate {
            TrackPredicate::Indices(indices) => indices.contains(&(index - 1)),
            TrackPredicate::Languages(language_ids) => {
                language_ids.is_match(&self.media.tracks[index].language)
            }
            TrackPredicate::Title(t) => t.is_match(&self.media.tracks[index].title),
            _ => true,
        }
    }

    /// Create a [`MediaFile] instance from a media file path.
    ///
    /// # Arguments
    ///
    /// * `fp` - The path to the media file.
    pub fn from_path(fp: &str) -> Option<Self> {
        if !utils::file_exists(fp) {
            return None;
        }

        logger::subsection(
            format!("File {}", UNIQUE_ID.fetch_add(0, Ordering::SeqCst) + 1),
            false,
        );
        logger::log_inline(
            format!("Extracting MediaInfo JSON data for file '{fp}'..."),
            false,
        );

        // Run the MediaInfo CLI process and grab the JSON output.
        let output = Command::new(&paths::PATHS.mediainfo)
            .arg("--Output=JSON")
            .arg(fp)
            .output();

        // Attempt to parse the JSON output.
        let json = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(e) => {
                logger::log(format!(" Error: {e}"), false);
                return None;
            }
        };

        logger::log(" Done.", false);

        // Were we able to successfully parse the output?
        if let Some(mut mf) = MediaFile::parse_json(&json) {
            mf.id = UNIQUE_ID.fetch_add(1, Ordering::SeqCst);

            // Set the media file path variable.
            mf.file_path = fp.to_string();

            // Do we have any attachments? If so, copy them to the main struct.
            mf.attachments = mf.media.tracks[0].extra_info.attachments.clone();

            // Set up the temporary directory structure for the file.
            mf.init_temp_directory();

            logger::log(format!("Total tracks: {}", mf.media.tracks.len()), false);
            logger::log(
                format!("Total attachments: {}", mf.attachments.len()),
                false,
            );

            // Return the MediaFile object.
            Some(mf)
        } else {
            None
        }
    }

    /// Get the path to the temporary folder for this media file.
    fn get_temp_path(&self) -> String {
        utils::join_path_segments(&paths::PATHS.temp, &[self.id.to_string()])
    }

    /// Get the path to the temporary folder for the given output type for this media file.
    ///
    /// # Arguments
    ///
    /// * `output_type` - The name of the subdirectory representing the output type.
    fn get_temp_for_output_type(&self, output_type: &str) -> String {
        utils::join_path_segments(&self.get_temp_path(), &[output_type])
    }

    /// Initialize the temporary directory structure for the media file.
    fn init_temp_directory(&self) -> bool {
        // Create each subdirectory.
        let mut result = true;
        for dir in ["attachments", "chapters", "tracks"] {
            let p = self.get_temp_for_output_type(dir);
            result &= fs::create_dir_all(p).is_ok();
        }

        result
    }

    /// Process a media file, applying any conversions and filters before remuxing the file.
    ///
    /// # Arguments
    ///
    /// * `out_path` - The path of the output media file.
    /// * `title` - The title of the media file.
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn process(&mut self, out_path: &str, title: &str, params: &UnifiedParams) -> bool {
        use crate::conversion_params::unified::DeletionOptions;

        self.output_path = out_path.to_string();

        // Set the file IDs of all child tracks.
        for track in &mut self.media.tracks {
            track.file_id = self.id;
        }

        // Filter the attachments based on the filter parameters.
        self.filter_internal_attachments(params);

        // Apply the default languages to tracks, if needed.
        self.apply_track_language_defaults(params);

        // Filter the tracks based on the filter parameters.
        // If the filtering is unsuccessful then we can't continue.
        if !self.filter_tracks(params) {
            return false;
        }

        // Extract the files.
        if !self.extract(
            true,
            params.attachments.import_from_original,
            params.chapters.import_from_original,
        ) {
            return false;
        }

        logger::log("", false);

        // Convert the audio tracks.
        if let Some(ac) = &params.audio_tracks.conversion {
            if ac.codec.is_some() && !self.convert_all_audio(ac) {
                return false;
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

        logger::log("", false);

        // Run any pre-muxing processes, if any were requested.
        self.run_commands(RunCommandType::PreMux, params);

        logger::log("", false);

        // Remux the media file.
        if self.remux_file(out_path, title, params) {
            logger::log("", false);

            // Run any post-muxing processes, if any were requested.
            self.run_commands(RunCommandType::PostMux, params);
        }

        logger::log("", false);

        // Delete the temporary files.
        match params.misc.remove_temp_files {
            Some(DeletionOptions::Delete) => {
                logger::log_inline("Attempting to delete temporary files... ", false);
                if utils::delete_directory(&self.get_temp_path()) {
                    logger::log(" files successfully deleted.", false);
                } else {
                    logger::log(" files could not be deleted.", false);
                }
            }
            Some(DeletionOptions::Trash) => {
                logger::log_inline("Attempting to delete temporary files... ", false);
                if trash::delete(self.get_temp_path()).is_ok() {
                    logger::log(" files successfully sent to the trash.", false);
                } else {
                    logger::log(" files could not be sent to the trash.", false);
                }
            }
            _ => {}
        }

        true
    }

    /// Add an attachment to the argument list.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `path` - A string slice representing the path to the attachment file.
    /// * `accepted_extensions` - A reference to the option containing permitted extensions list. If omitted then all extensions are permitted.
    fn add_attachment_if_matching(
        &self,
        args: &mut Vec<String>,
        path: &str,
        accepted_extensions: &Option<Vec<String>>,
    ) {
        let file_name = utils::get_file_name(path).unwrap_or_default();
        if file_name.is_empty() {
            return;
        }

        let valid_extensions = match accepted_extensions {
            Some(exts) => exts.clone(),
            None => Vec::new(),
        };

        // The file is a match if:
        //   * The extension is within the valid list, or the valid list is empty.
        //   * The extension is empty and the valid list is empty.
        let is_match = match utils::get_file_extension(&file_name) {
            Some(ext) => valid_extensions.is_empty() || valid_extensions.contains(&ext),
            None => valid_extensions.is_empty(),
        };

        if !is_match {
            return;
        }

        if !utils::file_exists(path) {
            logger::log(format!("[info] Attachment path '{path}' was selected for inclusion but the path couldn't be found. This may be expected if external commands have been used."), false);
            return;
        }

        // Set the attachment name.
        args.push("--attachment-name".to_string());
        args.push(file_name);

        // Set the attachment file path.
        args.push("--attach-file".to_string());
        args.push(path.to_string());
    }

    /// Apply the parameters related to any attachments to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_attachment_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        // Apply the internal (extracted) attachment muxing arguments, if needed.
        if params.attachments.import_from_original {
            self.apply_internal_attachment_mux_params(args, params);
        }

        // Add any external attachments from the specified folder, if needed.
        // We don't don't have any to add if the folder path is empty.
        let import_dir = params
            .attachments
            .import_from_folder
            .clone()
            .unwrap_or_default();
        if !import_dir.is_empty() {
            self.apply_external_attachment_mux_params(args, &import_dir, params);
        }
    }

    /// Apply the parameters related to any internal attachments to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_internal_attachment_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        // Iterate over all of the attachments.
        for attachment in &self.attachments {
            self.add_attachment_if_matching(
                args,
                &format!("{}/attachments/{attachment}", self.get_temp_path()),
                &params.attachments.import_original_extensions,
            );
        }
    }

    /// Apply the parameters related to any external attachments to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `dir` - The directory from which the files should be imported.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_external_attachment_mux_params(
        &self,
        args: &mut Vec<String>,
        dir: &String,
        params: &UnifiedParams,
    ) {
        // Read the contents of the import attachments folder recursively.
        for path in WalkDir::new(dir)
            .into_iter()
            .filter_map(MediaFile::filter_files)
        {
            // If the path is valid, add it to the kept attachments list.
            self.add_attachment_if_matching(
                args,
                &path,
                &params.attachments.import_folder_extensions,
            );
        }
    }

    /// Apply the parameters related the chapters to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_chapters_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        args.push("--chapter-language".to_string());
        args.push("en".to_string());

        // Did we export an existing chapters file?
        let chapters_fp =
            utils::join_path_segments(&self.get_temp_path(), &["chapters", "chapters.xml"]);
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
                format = interval;
            }

            args.push(format!("interval:{format}"));
        }
    }

    /// Apply any additional track parameters, such as default, forced, etc.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `track_id` - The ID of the track to which the parameters should be applied.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_additional_track_mux_params(
        &self,
        args: &mut Vec<String>,
        track_id: usize,
        params: &UnifiedParams,
    ) {
        // Do we have any track parameters to apply?
        let all_track_params = match &params.track_params {
            Some(tps) => tps,
            None => return,
        };

        // Do we have any parameters to apply to this track?
        let track_params = match all_track_params.iter().find(|t| t.id == track_id) {
            Some(tp) => tp,
            None => return,
        };

        let track_type = &self.media.tracks[track_id].track_type;

        let mut param_opts = HashMap::new();

        if let Some(b) = track_params.default {
            param_opts.insert("default-track", b);
        }
        if let Some(b) = track_params.enabled {
            param_opts.insert("track-enabled", b);
        }
        if let Some(b) = track_params.forced {
            if *track_type == TrackType::Subtitle {
                param_opts.insert("forced-display", b);
            } else {
                eprintln!("The forced flag was set for track ID {track_id}, but the track type does not support it.");
            }
        }
        if let Some(b) = track_params.hearing_impaired {
            if *track_type == TrackType::Audio {
                param_opts.insert("hearing-impaired", b);
            } else {
                eprintln!("The hearing impaired flag was set for track ID {track_id}, but the track type does not support it.");
            }
        }
        if let Some(b) = track_params.hearing_impaired {
            if *track_type == TrackType::Audio {
                param_opts.insert("visual-impaired", b);
            } else {
                eprintln!("The visually impaired flag was set for track ID {track_id}, but the track type does not support it.");
            }
        }
        if let Some(b) = track_params.text_descriptions {
            if *track_type == TrackType::Subtitle {
                param_opts.insert("text-descriptions", b);
            } else {
                eprintln!("The text descriptions flag was set for track ID {track_id}, but the track type does not support it.");
            }
        }
        if let Some(b) = track_params.original {
            param_opts.insert("original", b);
        }
        if let Some(b) = track_params.commentary {
            if matches!(*track_type, TrackType::Audio | TrackType::Subtitle) {
                param_opts.insert("commentary", b);
            } else {
                eprintln!("The commentary flag was set for track ID {track_id}, but the track type does not support it.");
            }
        }

        // Iterate over the specified parameters.
        for (k, v) in param_opts {
            args.push(format!("--{k}-flag"));
            args.push(format!("0:{}", utils::bool_to_yes_no(v)));
        }
    }

    /// Apply the parameters related the tracks to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_track_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        // Iterate over all of the tracks.
        for (i, track) in self.media.tracks.iter().enumerate() {
            let mut delay = track.delay;
            let mut delay_source = track.delay_source.clone();

            // Do we have a delay override for this track?
            if let Some(tp) = &params.track_params {
                if let Some(params) = tp.iter().find(|t| t.id == i) {
                    if let Some(d) = params.delay_override {
                        if delay_source == DelaySource::None {
                            delay_source = DelaySource::Container;
                        }
                        delay = d;
                    }
                }
            }

            // Do we need to specify a delay for the track?
            if delay != 0 {
                match delay_source {
                    DelaySource::Container => {
                        args.push("--sync".to_string());
                        args.push(format!("0:{}", track.delay));
                    }
                    DelaySource::None => {}
                    _ => {
                        todo!("DelaySource {delay_source:?} not yet implemented.");
                    }
                }
            }

            // Do we need to set the width and height?
            if track.width != 0 && track.height != 0 {
                args.push("--display-dimensions".to_string());
                args.push(format!("0:{}x{}", track.width, track.height));
            }

            // Do we need to set the bit depth?
            if track.bit_depth != 0 {
                args.push("--color-bits-per-channel".to_string());
                args.push(format!("0:{}", track.bit_depth));
            }

            // Apply any additional track parameters, if any were specified.
            self.apply_additional_track_mux_params(args, i, params);

            // Specify the track language. We set undefined for any video tracks.
            args.push("--language".to_string());
            if track.track_type == TrackType::Video {
                args.push("0:und".to_string());
            } else {
                args.push(format!("0:{}", track.language));
            }

            // Set the file path.
            args.push(format!("./tracks/{}", track.get_out_file_name()));
        }
    }

    /// Apply the parameters related the tags to be added to the media file.
    ///
    /// # Arguments
    ///
    /// * `args` - A reference to the vector containing the argument list.
    /// * `params` - The conversion parameters to be applied to the media file.
    fn apply_tag_mux_params(&self, args: &mut Vec<String>, params: &UnifiedParams) {
        let path = params.misc.tags_path.clone().unwrap_or_default();
        if !utils::file_exists(&path) {
            return;
        }

        // Set the global tags argument.
        args.push("--global-tags".to_string());
        args.push(path);
    }

    /// Filter files from a [`DirEntry`] iterator filter_map.
    ///
    /// # Arguments
    ///
    /// * `entry` - The [`DirEntry`] we are currently examining.
    ///
    /// # Returns
    ///
    /// A [`String`] if the [`DirEntry`] points to a file, and if the path is valid, None otherwise.
    fn filter_files(entry: Result<DirEntry, Error>) -> Option<String> {
        let dir_entry = match entry {
            Ok(de) => de,
            Err(_) => {
                return None;
            }
        };

        let path = dir_entry.path();
        if path.is_file() {
            Some(path.display().to_string())
        } else {
            None
        }
    }

    /// Run any pre-muxing commands.
    ///
    /// # Arguments
    ///
    /// * `run_type` - The type of command to be run.
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn run_commands(&self, run_type: RunCommandType, params: &UnifiedParams) {
        logger::log_inline("Checking for run commands... ", false);

        let run = match params.misc.run.clone() {
            Some(r) => r,
            None => {
                logger::log("no commands were specified.", false);
                return;
            }
        };

        let commands: Vec<&ProcessRun> = run
            .iter()
            .filter(|f| match run_type {
                RunCommandType::PreMux => matches!(f, ProcessRun::PreMux(_)),
                RunCommandType::PostMux => matches!(f, ProcessRun::PostMux(_)),
            })
            .collect();
        if commands.is_empty() {
            logger::log(
                format!("no command of type '{run_type:?}' was specified."),
                false,
            );
            return;
        }

        logger::log("\n", false);
        for command in commands {
            logger::log(
                format!("A command of type '{run_type:?}' was specified and will now be executed."),
                false,
            );

            let command_args = match command {
                ProcessRun::PreMux(args) => args,
                ProcessRun::PostMux(args) => args,
            };

            // The path to the command must always be the first in the list.
            // Everything that follows will be assumed to be arguments
            // to be passed to whatever command is being run.
            let path = &command_args[0];
            if !utils::file_exists(path) {
                logger::log(
                    format!(
                        "Run command of type {run_type:?} was specified, but the path doesn't exist!",
                    ),
                    false,
                );
                return;
            }

            // Go through the arguments list and replace any special parameters.
            // Currently there is only one, but there might eventually be more.
            let mut args: Vec<String> = command_args[1..].to_vec();
            for arg in &mut args {
                *arg = arg.replace("%i%", &self.file_path);
                *arg = arg.replace("%o%", &self.output_path);
                *arg = arg.replace("%t%", &self.get_temp_path());
            }

            // Run the command and show the results.
            match Command::new(path).args(args).output() {
                Ok(o) => {
                    logger::log(
                        "The command was successfully executed and yielded the following output:",
                        false,
                    );
                    let str = String::from_utf8_lossy(&o.stdout);
                    for line in str.split('\n') {
                        logger::log(format!(">\t{line}"), false);
                    }
                }
                Err(e) => {
                    logger::log(
                        format!("The command was not successfully executed and yielded the following output: {e:?}"), false
                    );
                }
            }
        }
    }

    /// Remux the attachments, chapters and tracks into a single file.
    ///
    /// # Arguments
    ///
    /// * `out_path` - The path to the expected location of the output media file.
    /// * `title` - The title of the media file.
    /// * `params` - The conversion parameters to be applied to the media file.
    pub fn remux_file(&self, out_path: &str, title: &str, params: &UnifiedParams) -> bool {
        logger::log("Remuxing media file... ", false);

        let mut args = Vec::with_capacity(100);

        // The output file path.
        args.push("-o".to_string());
        args.push(out_path.to_string());

        // The title of the media file, if needed.
        if let Some(b) = params.misc.set_file_title {
            if b {
                args.push("--title".to_string());
                args.push(title.to_string());
            }
        }

        // Apply the track muxing arguments.
        self.apply_track_mux_params(&mut args, params);

        // Apply the attachment muxing arguments, if needed.
        self.apply_attachment_mux_params(&mut args, params);

        // Apply the chapter muxing arguments, if needed.
        if params.chapters.import_from_original || params.chapters.create_if_not_present {
            self.apply_chapters_mux_params(&mut args, params);
        }

        // Apply the tag muxing arguments, if needed.
        if params.misc.tags_path.is_some() {
            self.apply_tag_mux_params(&mut args, params);
        }

        // Set the track order.
        let order = (0..self.media.tracks.len())
            .map(|i| format!("{i}:0"))
            .collect::<Vec<String>>()
            .join(",");

        args.push("--track-order".to_string());
        args.push(order);

        // Run the MKV merge process.
        let success = match mkvtoolnix::run_merge(&self.get_temp_path(), &args) {
            0 | 1 => {
                logger::log("Remuxing complete.", false);
                true
            }
            2 => {
                logger::log("Remuxing failed.", false);
                false
            }
            _ => true,
        };

        // Output the mkvmerge parameters, if the debug flag is set.
        if DEBUG_PARAMS {
            logger::log(
                format!(
                    "mkvmerge command line: \"{}\" {}",
                    mkvtoolnix::get_exe("mkvmerge"),
                    &args.join(" ")
                ),
                false,
            );
        }

        success
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

        match serde_json::from_str::<MediaFile>(json) {
            Ok(mi) => Some(mi),
            Err(e) => {
                logger::log(format!("Error attempting to parse JSON data: {e:?}"), true);
                None
            }
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

    /// The number of channels in the track. Only applicable to audio tracks.
    #[serde(rename = "Channels", deserialize_with = "string_to_u32", default)]
    pub channels: u32,

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

    /// The track's title. If this is not defined then an empty string will be used instead.
    #[serde(rename = "Title", default)]
    pub title: String,

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

    /// The width of the track, only applicable to video tracks.
    #[serde(rename = "Width", deserialize_with = "string_to_u32", default)]
    pub width: u32,

    /// The height of the track, only applicable to video tracks.
    #[serde(rename = "Height", deserialize_with = "string_to_u32", default)]
    pub height: u32,

    /// The height of the track, only applicable to video tracks.
    #[serde(rename = "BitDepth", deserialize_with = "string_to_u32", default)]
    pub bit_depth: u32,

    /// The additional track information.
    ///
    /// `Note:` This field will only contains meaningful data when the track type is [`TrackType::General`].
    #[serde(rename = "extra", default)]
    pub extra_info: MediaInfoExtra,

    /// The index of the file to which this track belongs.
    #[serde(skip)]
    pub file_id: usize,

    /// Should this file be kept when remuxing?
    #[serde(skip)]
    pub keep: usize,
}

impl MediaFileTrack {
    /// Get the output name for this track.
    pub fn get_out_file_name(&self) -> String {
        let ext = MediaFileTrack::get_extension_from_codec(&self.codec);

        format!("{}_{}_{}.{ext}", self.track_type, self.id, self.language)
    }

    /// Get the path to the extracted (original) media file.
    pub fn get_input_file_path(&self) -> String {
        // Determine the output file name.
        format!(
            "{}\\tracks\\{}",
            utils::join_path_segments(&paths::PATHS.temp, &[self.file_id.to_string()]),
            self.get_out_file_name()
        )
    }

    /// Get the expected path to the re-encoded media file.
    ///
    /// # Arguments
    ///
    /// * `out_codec` - The requested output codec.
    pub fn get_output_file_path(&self, out_codec: &Codec) -> String {
        // Get the new file extension and set the output file extension to it.
        let out_ext = MediaFileTrack::get_extension_from_codec(out_codec);
        utils::swap_file_extension(&self.get_input_file_path(), &out_ext)
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
            Codec::Eac3 => "eac3",
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

    Ok(string.split(" / ").map(|s| s.to_string()).collect())
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
        | "A_AAC/MPEG4/LTP" | "A_AAC-1" | "A_AAC-2" => Codec::Aac,
        "A_QUICKTIME" | "A_QUICKTIME/QDMC" | "A_QUICKTIME/QDM2" => Codec::QuickTime,
        "A_TTA1" => Codec::TheTrueAudio,
        "A_WAVPACK4" => Codec::WavPack4,
        "A_EAC3" => Codec::Eac3,

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
            println!("Unexpected codec ID when parsing MKV file: {string}");
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

    // We specifically want to map tracks with an unspecified language
    // to und, to avoid them being missed.
    if string.is_empty() {
        Ok(default_track_language())
    } else {
        Ok(string)
    }
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
            Ok((n * 1000.0) as i32)
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
    "und".to_string()
}
