use crate::{
    conversion_params::unified::UnifiedParams, input_profile::InputProfile, logger, utils,
};

use lexical_sort::{natural_cmp, StringSort};
use serde_derive::Deserialize;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Instant,
};

const VALID_EXTENSIONS: [&str; 1] = ["mkv"];

#[derive(Clone, Copy, Deserialize)]
pub enum PadType {
    One,
    Ten,
    Hundred,
    Thousand,
}

pub struct FileProcessor {
    pub input_paths: Vec<String>,
    pub output_paths: Vec<String>,
    pub titles: Vec<String>,
}

impl FileProcessor {
    pub fn new(profile: &InputProfile) -> Option<Self> {
        logger::section("File Processing Initialization", false);

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

        // If one or more required paths were invalid then we can't continue.
        if !check {
            return None;
        }

        let mut input_paths = Vec::new();
        let mut output_paths = Vec::new();
        let mut titles = Vec::new();

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
                return None;
            }
        };

        // Create a local copy of the substitution instance.
        let mut substitutions = profile.substitutions.clone();

        // If we have a stop clause then we are permitted to have
        // less files than specified in the final list, but not more.
        let mut has_stop_clause = false;

        // Iterate over each line of the file.
        let mut index = profile.start_from.unwrap_or_default();
        for line in BufReader::new(file).lines() {
            // This can occur if the line does not contain valid UTF-8
            // sequences.
            if let Err(e) = line {
                logger::log(format!("Error parsing input names file: {e}"), false);
                continue;
            }

            let line = &line.unwrap();

            // If the STOP clause is present then we should stop reading
            // the file name lines.
            if line == "###STOP###" {
                has_stop_clause = true;
                break;
            }

            // Sanitize the title of the media file based on the supplied
            // substitution parameters.
            let sanitized = substitutions.apply(line);

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
            output_paths.push(utils::join_path_segments(&profile.output_dir, &[file_name]));

            // Add the title to the vector.
            titles.push(sanitized.to_string());

            // Increment the index counter.
            index += 1;
        }

        logger::log(
            format!(
                "{} file names are present in the output file name list",
                output_paths.len()
            ),
            false,
        );

        // Build the list of input file paths.
        let read = fs::read_dir(&profile.input_dir);
        if let Ok(dir) = read {
            for entry in dir.flatten() {
                let path = entry.path();

                if let Some(ext) = path.extension() {
                    // We always want to check extensions in lowercase.
                    if VALID_EXTENSIONS.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                        input_paths.push(path.display().to_string());
                    }
                }
            }
        } else {
            logger::log(
                format!("Failed to read input files directory: {read:?}"),
                true,
            );
            panic!();
        }

        // Do we have any files in the input directory?
        if input_paths.is_empty() {
            logger::log(
                "There are no applicable files in the input directory.",
                true,
            );
            return None;
        }

        // Sort the input file paths using a natural sorting algorithm.
        input_paths.string_sort_unstable(natural_cmp);

        logger::log(
            format!(
                "{} applicable files are present in the input files directory",
                input_paths.len()
            ),
            false,
        );

        // If the stop clause has been specified then we need to truncate
        // the input file list to be the same length as the output file list.
        if has_stop_clause {
            input_paths.truncate(output_paths.len());
        }

        // We must now check that the number of files in the input
        // directory is equal to the number of entries from the output file list.
        if input_paths.len() != output_paths.len() {
            logger::log(
                format!(
                    "The number of files in the input directory {} is not equal to the number of files in the output file list {}",
                    input_paths.len(),
                    output_paths.len()
                ),
                true,
            );
            return None;
        }

        logger::log(
            "The number of files in the input directory and output list match.",
            false,
        );

        Some(Self {
            input_paths,
            output_paths,
            titles,
        })
    }

    /// Build a filename from a name, an index (optional) and a pad type (optional).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the file.
    /// * `index` - The index of the file, if applicable.
    /// * `pad_type` - The pad type to be applied to the index, if applicable.
    fn file_name_from_padded_index(name: &str, index: usize, pad_type: Option<PadType>) -> String {
        let mut str = if let Some(pad) = pad_type {
            match pad {
                PadType::One => {
                    format!("{index} - {name}")
                }
                PadType::Ten => {
                    format!("{index:02} - {name}")
                }
                PadType::Hundred => {
                    format!("{index:03} - {name}")
                }
                PadType::Thousand => {
                    format!("{index:04} - {name}")
                }
            }
        } else {
            name.to_string()
        };

        str.push_str(".mkv");
        str
    }

    /// Process each of the media files in the input directory.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters to be used while processing the media files.
    ///
    pub fn process(&self, params: &UnifiedParams) {
        use crate::{conversion_params::unified::DeletionOptions, media_file::MediaFile};
        use system_shutdown::shutdown;

        logger::section("Setup", false);

        let now = Instant::now();

        // Process the data from each of the media files.
        let media_len = self.input_paths.len();
        let mut media = Vec::with_capacity(media_len);
        for i in 0..media_len {
            if let Some(mf) = MediaFile::from_path(&self.input_paths[i]) {
                media.push(mf);
            }
        }

        logger::splitter(false);
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
            logger::subsection(&format!("File {} of {}", i + 1, media_len), true);

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

            // Delete the original file, if required.
            if let Some(del) = &params.misc_params.remove_original_file {
                match del {
                    DeletionOptions::Delete => {
                        logger::log_inline("Attempting to delete original media file... ", false);
                        if fs::remove_file(&self.input_paths[i]).is_ok() {
                            logger::log(" file successfully deleted.", false);
                        } else {
                            logger::log(" file could not be deleted.", false);
                        }
                    }
                    DeletionOptions::Trash => {
                        logger::log_inline("Attempting to delete original media file... ", false);
                        if trash::delete(&self.input_paths[i]).is_ok() {
                            logger::log(" file successfully sent to the trash.", false);
                        } else {
                            logger::log(" file could not be sent to the trash.", false);
                        }
                    }
                    _ => {}
                }
            }
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

        // Shutdown the computer after processing, if required.
        if params.misc_params.shutdown_upon_completion {
            match shutdown() {
                Ok(_) => logger::log("Attempting to shutdown down the computer...", true),
                Err(e) => logger::log(format!("Failed to shutdown the computer: {e}"), true),
            }
        }
    }
}
