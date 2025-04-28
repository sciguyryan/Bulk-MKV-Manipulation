use crate::{
    conversion_params::unified::{DeletionOptions, UnifiedParams},
    converters,
    input_profile::InputProfile,
    logger,
    media_file::MediaFile,
    utils,
};

use lexical_sort::{natural_cmp, StringSort};
use serde_derive::Deserialize;
use std::{
    fs::{self, DirEntry, File},
    io::{BufRead, BufReader, Error},
    time::Instant,
};

/// A list of valid extension that can be automatically processed.
const VALID_EXTENSIONS: [&str; 1] = ["mkv"];
/// A list of valid extensions that can be automatically converted into MKV files in pre-processing.
const VALID_REMUX_EXTENSIONS: [&str; 1] = ["mp4"];
/// The file list early stop clause.
const STOP_CLAUSE: &str = "###STOP###";

#[derive(Clone, Copy, Deserialize)]
pub enum PadType {
    One,
    Ten,
    Hundred,
    Thousand,
}

pub struct FileProcessor {
    pub has_stop_clause: bool,
    pub input_paths: Vec<String>,
    pub output_paths: Vec<String>,
    pub titles: Vec<String>,
}

impl FileProcessor {
    pub fn new(profile: &InputProfile) -> Option<Self> {
        logger::section("File Processing Initialization", false);

        let mut s = Self {
            has_stop_clause: false,
            input_paths: vec![],
            output_paths: vec![],
            titles: vec![],
        };

        // If one or more required paths were invalid then we can't continue.
        if !FileProcessor::validate_paths(profile) {
            return None;
        }

        // Build the output file name list.
        s.build_output_list(profile);
        if s.output_paths.is_empty() || s.titles.is_empty() {
            return None;
        }

        // Build the list of input file paths.
        s.build_input_list(profile);
        if s.input_paths.is_empty() {
            return None;
        }

        // We must now check that the number of files in the input
        // directory is equal to the number of entries from the output file list.
        if s.input_paths.len() != s.output_paths.len() {
            logger::log(
                format!(
                    "The number of files in the input directory {} is not equal to the number of files in the output file list {}",
                    s.input_paths.len(),
                    s.output_paths.len()
                ),
                true,
            );
            return None;
        }

        logger::log(
            "The number of files in the input directory and output list match.",
            false,
        );

        Some(s)
    }

    /// Build the input file list from the parameter specified by the [`InputProfile`].
    ///
    /// # Arguments
    ///
    /// * `profile` - The [`InputProfile`] specified when running the program.
    fn build_input_list(&mut self, profile: &InputProfile) {
        let read = fs::read_dir(&profile.input_dir);
        assert!(
            read.is_ok(),
            "Failed to read input files directory: {read:?}"
        );

        // Add all of the matching files into the file list.
        for path in read
            .unwrap()
            .filter_map(|p| FileProcessor::filter_by_file_extension(p, &VALID_EXTENSIONS))
        {
            self.input_paths.push(path);
        }

        // Do we have any files in the input directory?
        if self.input_paths.is_empty() {
            logger::log(
                "There are no applicable files in the input directory.",
                true,
            );
            return;
        }

        // Sort the input file paths using a natural sorting algorithm.
        self.input_paths.string_sort_unstable(natural_cmp);

        logger::log(
            format!(
                "{} applicable file{} present in the input files directory.",
                self.input_paths.len(),
                if self.output_paths.len() != 1 {
                    "s are"
                } else {
                    " is"
                }
            ),
            false,
        );

        // If the stop clause has been specified then we need to truncate
        // the input file list to be the same length as the output file list.
        if self.has_stop_clause {
            self.input_paths.truncate(self.output_paths.len());
        }
    }

    /// Build the output file list from the parameter specified by the [`InputProfile`].
    ///
    /// # Arguments
    ///
    /// * `profile` - The [`InputProfile`] specified when running the program.
    fn build_output_list(&mut self, profile: &InputProfile) {
        // Read the file containing the output names.
        let file = match File::open(&profile.output_names_file_path) {
            Ok(f) => f,
            Err(e) => {
                logger::log(
                    format!(
                        "An error occurred while attempting to open the output names file: {e:?}"
                    ),
                    true,
                );
                return;
            }
        };

        // Create a local copy of the substitution instance.
        let mut substitutions = profile.substitutions.clone();

        // Iterate over each line of the file.
        let mut index = profile.start_from.unwrap_or_default();
        for line in BufReader::new(file).lines() {
            // This can occur if the line does not contain valid UTF-8
            // sequences.
            if let Err(e) = line {
                logger::log(format!("Error parsing input names file: {e}"), false);
                continue;
            }

            let line = line.unwrap();

            // If the STOP clause is present then we should stop reading
            // the file name lines.
            if line == STOP_CLAUSE {
                self.has_stop_clause = true;
                break;
            }

            // Sanitize the title of the media file based on the supplied
            // substitution parameters.
            let sanitized = substitutions.apply(&line);

            // Skip empty lines and comment lines.
            if sanitized.is_empty() || sanitized.starts_with('#') {
                continue;
            }

            // Handle the number padding, if required.
            let file_name = FileProcessor::file_name_from_padded_index(
                &sanitized,
                index,
                profile.index_pad_type,
            );

            // Add the file output path to the vector.
            self.output_paths
                .push(utils::join_path_segments(&profile.output_dir, &[file_name]));

            // Add the title to the vector.
            self.titles.push(sanitized.to_string());

            // Increment the index counter.
            index += 1;
        }

        logger::log(
            format!(
                "{} file name{} are present in the output file name list.",
                self.output_paths.len(),
                if self.output_paths.len() != 1 {
                    "s"
                } else {
                    ""
                }
            ),
            false,
        );

        if let Some(b) = &profile.processing_params.misc.pre_mux_media_files {
            if *b {
                // Remux certain other media files to allow them to be automatically handled.
                FileProcessor::pre_mux_media_files(profile);
            }
        }
    }

    /// Build a filename from a name, an index (optional) and a pad type (optional).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the file.
    /// * `index` - The index of the file, if applicable.
    /// * `pad_type` - An option containing the [`PadType`] to be applied to the index.
    ///
    /// # Returns
    ///
    /// A String giving the new output file name.
    fn file_name_from_padded_index(name: &str, index: usize, pad_type: Option<PadType>) -> String {
        let mut str = match pad_type {
            Some(PadType::One) => {
                format!("{index} – {name}")
            }
            Some(PadType::Ten) => {
                format!("{index:02} – {name}")
            }
            Some(PadType::Hundred) => {
                format!("{index:03} – {name}")
            }
            Some(PadType::Thousand) => {
                format!("{index:04} – {name}")
            }
            None => name.to_string(),
        };

        str.push_str(".mkv");
        str
    }

    /// Filter a [`DirEntry`] based on whether it is a file, and has a specific extension.
    ///
    /// # Arguments
    ///
    /// * `entry` - A reference to the [`DirEntry`] object.
    ///
    /// # Returns
    ///
    /// A String giving the path to the file, if its extension is within the valid extensions list.
    fn filter_by_file_extension(entry: Result<DirEntry, Error>, exts: &[&str]) -> Option<String> {
        // Eliminate invalid entries.
        let dir_entry = match entry {
            Ok(de) => de,
            Err(_) => {
                return None;
            }
        };

        // We are only interested in files.
        let path = dir_entry.path();
        if !path.is_file() {
            return None;
        }

        // Check if the file has an extension, and if the extension is within the
        // valid extensions slice.
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .to_string();

        if exts.contains(&extension.as_str()) {
            Some(path.display().to_string())
        } else {
            None
        }
    }

    /// Handle the removal of the the original media file, if remuxing has taken place.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    /// * `params` - The [`UnifiedParams`] to be used while processing the media files.
    fn maybe_delete_original_file(path: &str, params: &UnifiedParams) {
        match params.misc.remove_original_file {
            Some(DeletionOptions::Delete) => {
                logger::log_inline("Attempting to delete original media file... ", false);
                if fs::remove_file(path).is_ok() {
                    logger::log(" file successfully deleted.", false);
                } else {
                    logger::log(" file could not be deleted.", false);
                }
            }
            Some(DeletionOptions::Trash) => {
                logger::log_inline("Attempting to delete original media file... ", false);
                if trash::delete(path).is_ok() {
                    logger::log(" file successfully sent to the trash.", false);
                } else {
                    logger::log(" file could not be sent to the trash.", false);
                }
            }
            _ => {}
        }
    }

    /// Handle the removal of the the original media file, if remuxing has taken place.
    ///
    /// # Arguments
    ///
    /// * `params` - The [`UnifiedParams`] to be used while processing the media files.
    fn maybe_shutdown(params: &UnifiedParams) {
        // Shutdown the computer after processing, if required.
        if let Some(b) = params.misc.shutdown_upon_completion {
            if b {
                match system_shutdown::shutdown() {
                    Ok(_) => logger::log("Attempting to shutdown down the computer...", true),
                    Err(e) => logger::log(format!("Failed to shutdown the computer: {e}"), true),
                }
            }
        }
    }

    /// Run a pre-processing remux on certain media files within the input directory
    /// to permit them to be correctly handled by the main remuxing system.
    ///
    /// # Arguments
    ///
    /// * `profile` - The [`InputProfile`] specified when running the program.
    fn pre_mux_media_files(profile: &InputProfile) {
        logger::log(
            "Running pre-mux for files within the input directory...",
            false,
        );

        // Build the list of input file paths.
        let read = fs::read_dir(&profile.input_dir);
        assert!(
            read.is_ok(),
            "Failed to read input files directory: {read:?}"
        );

        // Add all of the matching files into the file list.
        for path in read
            .unwrap()
            .filter_map(|p| FileProcessor::filter_by_file_extension(p, &VALID_REMUX_EXTENSIONS))
        {
            logger::log(
                format!("File \"{path}\" is a valid remuxing target and will be remuxed..."),
                false,
            );

            let out_path = utils::swap_file_extension(&path, "mkv");
            converters::remux_media_file(&path, &out_path);

            // Delete the original file, if required.
            MediaFile::delete_path(&path, &profile.processing_params.misc.remove_original_file);
        }
    }

    /// Process each media file in the input directory.
    ///
    /// # Arguments
    ///
    /// * `params` - The [`UnifiedParams`] to be used while processing the media file.
    pub fn process(&self, params: &UnifiedParams) {
        logger::section("Setup", false);

        let now = Instant::now();

        // Process the data from each of the media files.
        let mut media: Vec<MediaFile> = self
            .input_paths
            .iter()
            .filter_map(|p| MediaFile::from_path(p))
            .collect();

        logger::log("", false);
        logger::log(
            format!(
                "Setup complete, in {}.",
                utils::format_duration(now.elapsed().as_secs())
            ),
            false,
        );

        logger::section("File Processing", true);

        // Process each media file.
        let mut success = true;
        for (i, m) in &mut media.iter_mut().enumerate() {
            logger::subsection(
                format!("File {} of {}", i + 1, self.input_paths.len()),
                true,
            );

            let start = Instant::now();
            if !m.process(&self.output_paths[i], &self.titles[i], params) {
                logger::log("Processing failed.", true);
                success = false;
                break;
            }

            logger::log(
                format!(
                    "Processing complete, in {}.",
                    utils::format_duration(start.elapsed().as_secs())
                ),
                true,
            );

            FileProcessor::maybe_delete_original_file(&self.input_paths[i], params);
        }

        logger::section("", true);
        if success {
            logger::log("All files have been successfully processed!", true);
        } else {
            logger::log(
                "One or more errors occurred and the files could not be processed.",
                true,
            );
        }

        FileProcessor::maybe_shutdown(params);
    }

    /// Validate the paths specified by the [`InputProfile`] are valid.
    ///
    /// # Arguments
    ///
    /// * `profile` - The [`InputProfile`] specified when running the program.
    fn validate_paths(profile: &InputProfile) -> bool {
        let mut check = true;
        if !utils::dir_exists(&profile.input_dir) {
            logger::log(
                format!("Input directory '{}' does not exist", profile.input_dir),
                true,
            );
            check = false;
        }

        if !utils::dir_exists(&profile.output_dir) {
            logger::log(
                format!("Output directory '{}' does not exist", profile.output_dir),
                true,
            );
            check = false;
        }

        if !utils::file_exists(&profile.output_names_file_path) {
            logger::log(
                format!(
                    "Output file names file '{}' does not exist",
                    profile.output_names_file_path
                ),
                true,
            );
            check = false;
        }

        check
    }
}
