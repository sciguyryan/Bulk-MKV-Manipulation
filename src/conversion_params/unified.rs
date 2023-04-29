use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;

use super::{audio::AudioParams, subtitle::SubtitleParams, video::VideoParams};

#[derive(Deserialize)]
pub struct UnifiedParams {
    /// Parameters related to the audio tracks.
    pub audio_tracks: UnifiedAudioParams,
    /// Parameters related to the subtitle tracks.
    pub subtitle_tracks: UnifiedSubtitleParams,
    /// Parameters related to the video tracks.
    pub video_tracks: UnifiedVideoParams,
    /// Parameters related to any other type of track.
    pub other_tracks: UnifiedOtherTrackParams,
    /// Parameters related to track ordering, forced tracks, etc.
    pub track_params: Option<Vec<TrackParams>>,
    /// Parameters related to the attachments.
    pub attachments: AttachmentParams,
    /// Parameters related to the chapters.
    pub chapters: ChapterParams,
    // Various other parameters that do not fit into another category.
    pub misc: MiscParams,
}

#[derive(Deserialize, Clone)]
pub struct TrackParams {
    /// The ID of the track to which these parameters should be applied.
    pub id: usize,
    /// Should the track be set as default?
    pub default: Option<bool>,
    /// Should the track be set as enabled? If unset, the track will be enabled by default.
    pub enabled: Option<bool>,
    /// Should the track be set as forced?
    pub forced: Option<bool>,
    /// Should the track be set as being for the hearing impaired?
    pub hearing_impaired: Option<bool>,
    /// Should the track be set as being for the visual impaired?
    pub visual_impaired: Option<bool>,
    /// Should the track be set being for text descriptions?
    pub text_descriptions: Option<bool>,
    /// Is the track in the original content's language?
    pub original: Option<bool>,
    /// Does the track contain commentary?
    pub commentary: Option<bool>,
    /// Should the delay be overridden for this track?
    pub delay_override: Option<i32>,
}

#[derive(Deserialize)]
pub struct AttachmentParams {
    /// Should attachments be imported from the original file?
    /// Any existing attachments will be exported and included in the final file.
    pub import_from_original: bool,
    /// The list of attachment extensions to be included from the original file.
    /// An empty list will indicate that all files should be included.
    #[serde(deserialize_with = "array_to_lowercase_string_vec")]
    pub import_original_extensions: Option<Vec<String>>,
    /// The path to a folder from which all files should be imported
    /// as attachments.
    pub import_from_folder: Option<String>,
    /// The list of attachment extensions to be included from the folder.
    /// An empty list will indicate that all files should be included.
    #[serde(deserialize_with = "array_to_lowercase_string_vec")]
    pub import_folder_extensions: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ChapterParams {
    /// Should chapters be imported from the original file?
    /// Any existing chapters will be exported and included in the final file.
    pub import_from_original: bool,
    /// Should chapters be created, if no chapters are present?
    pub create_if_not_present: bool,
    /// The interval for creating chapters, must be in the following format: HH:MM:SS.nnnnnnnnn.
    /// If `None` is specified then chapters will be created at 5 minute intervals.
    pub create_interval: Option<String>,
}

#[derive(Default, Deserialize, PartialEq, Eq, Clone)]
pub enum DeletionOptions {
    /// Delete the file using the default method.
    Delete,
    /// Delete the file by sending it to the trash, if possible.
    Trash,
    /// Do not delete the file.
    #[default]
    None,
}

#[derive(Deserialize)]
pub struct MiscParams {
    /// The method to be used when removing the original files, if specified.
    pub remove_original_file: Option<DeletionOptions>,
    /// The method to be used when removing the temporary files, if specified.
    pub remove_temp_files: Option<DeletionOptions>,
    // Should the title of the media file be set?
    pub set_file_title: Option<bool>,
    /// Should the computer be shutdown after the processing is complete?
    pub shutdown_upon_completion: Option<bool>,
    /// The path to the tags file.
    /// If the path is not specified, or is invalid, then no tags will be added.
    pub tags_path: Option<String>,
    /// The paths to any processes that should be run before or after this processing step.
    pub run: Option<Vec<ProcessRun>>,
}

pub trait PredicateFilterMatch<T> {
    fn is_match(&self, needle: T) -> bool;
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
pub enum ProcessRun {
    // A command to be run prior to muxing.
    PreMux(Vec<String>),
    // A command to be run after muxing.
    PostMux(Vec<String>),
}

#[derive(Deserialize)]
pub struct TrackIndexPredicate {
    ids: Vec<usize>,
}

impl TrackIndexPredicate {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

impl PredicateFilterMatch<usize> for TrackIndexPredicate {
    /// Checks to see if a given track language ID is a match against the specified filters.
    ///
    /// # Returns
    ///
    /// True if track language ID was a match for the filters, false otherwise.
    fn is_match(&self, needle: usize) -> bool {
        self.ids.is_empty() || self.ids.contains(&needle)
    }
}

#[derive(Deserialize)]
pub struct TrackTitlePredicate {
    /// The predicate filter type.
    filter_condition: TrackTitlePredicateCondition,
    /// The predicate filter strings.
    filters: Vec<TrackTitlePredicateType>,
    /// The predicate regular expression objects, if defined.
    #[serde(skip)]
    regex_filters: Vec<Regex>,
}

impl TrackTitlePredicate {
    /// Attempt to initialize any regular expression objects that have been defined via a filters.
    ///
    /// # Returns
    ///
    /// True if the regular expression were valid, false otherwise.
    pub fn initialize_regex(&mut self) -> bool {
        for entry in &self.filters {
            if let TrackTitlePredicateType::Regex(s) = (*entry).clone() {
                let r = Regex::new(&s);
                if let Ok(re) = r {
                    self.regex_filters.push(re);
                } else {
                    eprintln!("An error occurred while initializing regex: {r:?}");
                    return false;
                }
            }
        }

        true
    }

    /// Check if a given string is a match for the track title.
    ///
    /// # Returns
    ///
    /// True if the needle string is a match for the track title, false otherwise.
    fn is_text_match(&self, needle: &str) -> bool {
        let mut is_overall_match = true;

        for f in &self.filters {
            let is_sub_match = match f {
                TrackTitlePredicateType::Contains(s) => needle.contains(s),
                TrackTitlePredicateType::Equals(s) => s == needle,
                _ => continue,
            };

            match self.filter_condition {
                TrackTitlePredicateCondition::And => {
                    is_overall_match &= is_sub_match;
                }
                TrackTitlePredicateCondition::Or => {
                    is_overall_match |= is_sub_match;
                }
                TrackTitlePredicateCondition::Not => {
                    is_overall_match &= !is_sub_match;
                }
            }

            if !is_overall_match {
                break;
            }
        }

        is_overall_match
    }

    /// Check if a given string is a regular expression match for the track title.
    ///
    /// # Returns
    ///
    /// True if the needle string is a regular expression match for the track title, false otherwise.
    fn is_regex_match(&self, needle: &str) -> bool {
        let mut is_overall_match = true;

        for r in &self.regex_filters {
            let is_sub_match = r.is_match(needle);

            match self.filter_condition {
                TrackTitlePredicateCondition::And => {
                    is_overall_match &= is_sub_match;
                }
                TrackTitlePredicateCondition::Or => {
                    is_overall_match |= is_sub_match;
                }
                TrackTitlePredicateCondition::Not => {
                    is_overall_match &= !is_sub_match;
                }
            }

            if !is_overall_match {
                break;
            }
        }

        is_overall_match
    }
}

impl PredicateFilterMatch<&str> for TrackTitlePredicate {
    /// Checks to see if a given track title is a match against the specified filters.
    ///
    /// # Returns
    ///
    /// True if track title was a match for the filters, false otherwise.
    fn is_match(&self, needle: &str) -> bool {
        self.is_text_match(needle) && self.is_regex_match(needle)
    }
}

#[derive(Deserialize, Eq, PartialEq)]
pub enum TrackTitlePredicateCondition {
    /// If all of the filters are a match, then the title will be considered as matching.
    And,
    // If none of the filters are a match, then the title will be considered as matching.
    Not,
    /// If any of the filters are a match, then the title will be considered as matching.
    Or,
}

#[derive(Clone, Deserialize)]
pub enum TrackTitlePredicateType {
    /// A fuzzy match - a match will be counted if the title contains the string.
    Contains(String),
    /// A strict match - a match will be counted if the title exactly matches the string.
    Equals(String),
    /// A regular expression match - a match will be counted if the title matches the regular expression.
    Regex(String),
}

#[derive(Deserialize)]
pub struct TrackLanguagePredicate {
    /// A list of language ID codes that have been specified in the filters.
    pub ids: Vec<String>,
}

impl TrackLanguagePredicate {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

impl PredicateFilterMatch<&str> for TrackLanguagePredicate {
    /// Checks to see if a given track language ID is a match against the specified filters.
    ///
    /// # Returns
    ///
    /// True if track language ID was a match for the filters, false otherwise.
    fn is_match(&self, needle: &str) -> bool {
        self.ids.is_empty() || self.ids.contains(&needle.to_string())
    }
}

#[derive(Default, Deserialize)]
pub enum TrackPredicate {
    /// Filter by track indices.
    #[serde(rename = "index")]
    Index(TrackIndexPredicate),
    /// Filter by track language code.
    #[serde(rename = "language")]
    Language(TrackLanguagePredicate),
    /// Filter by track title.
    #[serde(rename = "title")]
    Title(TrackTitlePredicate),
    /// No filter should be applied.
    #[default]
    None,
}

impl TrackPredicate {
    /// Attempt to initialize any regular expression objects that have been defined via a filters.
    ///
    /// # Returns
    ///
    /// True if the regular expression were valid, false otherwise.
    pub fn initialize_regex(&mut self) -> bool {
        if let TrackPredicate::Title(tft) = self {
            tft.initialize_regex()
        } else {
            true
        }
    }
}

#[derive(Deserialize)]
pub struct UnifiedAudioParams {
    /// The type of filter that should be applied to this track.
    pub predicate: TrackPredicate,
    /// The conversion parameters for audio tracks.
    pub conversion: Option<AudioParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
    /// The number of tracks of this type to retain, in total.
    pub total_to_retain: Option<usize>,
}

#[derive(Deserialize)]
pub struct UnifiedSubtitleParams {
    /// The type of filter that should be applied to this track.
    pub predicate: TrackPredicate,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<SubtitleParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
    /// The number of tracks of this type to retain, in total.
    pub total_to_retain: Option<usize>,
}

#[derive(Deserialize)]
pub struct UnifiedOtherTrackParams {
    /// Should "other" be imported from the original file?
    /// These are tracks that are not of type: audio, subtitle or video.
    pub import_from_original: bool,
}

#[derive(Deserialize)]
pub struct UnifiedVideoParams {
    /// The type of filter that should be applied to this track.
    pub predicate: TrackPredicate,
    /// The conversion parameters for subtitle tracks.
    pub conversion: Option<VideoParams>,
    /// If the language is undefined, what should the language be
    /// assumed as being?
    pub default_language: Option<String>,
    /// The number of tracks of this type to retain, in total.
    pub total_to_retain: Option<usize>,
}

fn array_to_lowercase_string_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut vec: Vec<String> = Vec::deserialize(deserializer)?;

    for v in &mut vec {
        *v = v.to_lowercase()
    }

    Ok(Some(vec))
}
