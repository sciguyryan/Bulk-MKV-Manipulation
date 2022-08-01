use lexical_sort::{natural_cmp, StringSort};
use serde_derive::Deserialize;

use crate::{
    conversion_params::unified::UnifiedParams, input_profile::InputProfile, logger, utils,
};

use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Instant,
};

#[derive(Deserialize)]
#[allow(unused)]
pub enum PadType {
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

        if !utils::dir_exists(&profile.input_dir) {
            logger::log(
                &format!("Input directory '{}' does not exist", profile.input_dir),
                true,
            );
            return None;
        }

        if !utils::dir_exists(&profile.output_dir) {
            logger::log(
                &format!("Output directory '{}' does not exist", profile.output_dir),
                true,
            );
            return None;
        }

        if !utils::file_exists(&profile.output_names_file_path) {
            logger::log(
                &format!(
                    "Output file names file '{}' does not exist",
                    profile.output_names_file_path
                ),
                true,
            );
            return None;
        }

        let mut input_paths = Vec::new();
        let mut output_paths = Vec::new();
        let mut titles = Vec::new();

        // Build the list of input file paths.
        let read = fs::read_dir(&profile.input_dir);
        if let Ok(dir) = read {
            for entry in dir.flatten() {
                let path = entry.path();
                let ext = path.extension();
                if ext.is_none() {
                    continue;
                }

                // We always want to check extensions in lowercase.
                let ext = ext.unwrap().to_string_lossy().to_lowercase();
                if ext != "mkv" {
                    continue;
                }

                input_paths.push(path.display().to_string());
            }
        } else {
            logger::log(
                &format!("Failed to read input files directory: {:?}", read),
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
            &format!(
                "{} applicable files are present in the input files directory",
                input_paths.len()
            ),
            false,
        );

        // Read the file containing the output names.
        let mut index = profile.start_from;
        let file = match File::open(&profile.output_names_file_path) {
            Ok(f) => f,
            Err(e) => {
                logger::log(
                    &format!(
                        "An error occurred while attempting to open the output names file: {:?}",
                        e
                    ),
                    true,
                );
                return None;
            }
        };

        // Create a local copy of the substitution instance.
        let mut substitutions = profile.substitutions.clone();

        // Iterate over each line of the file.
        for line in BufReader::new(file).lines().flatten() {
            // Sanitize the title of the media file based on the supplied
            // substitution parameters.
            let sanitized = substitutions.apply(&line);

            // Skip empty lines.
            if sanitized.is_empty() {
                continue;
            }

            // Handle the number padding.
            let file_name = match &profile.index_pad_type {
                PadType::Ten => {
                    format!("{:02} - {}.mkv", index, sanitized)
                }
                PadType::Hundred => {
                    format!("{:03} - {}.mkv", index, sanitized)
                }
                PadType::Thousand => {
                    format!("{:04} - {}.mkv", index, sanitized)
                }
            };

            // Add the file output path to the vector.
            output_paths.push(utils::join_path_segments(
                &profile.output_dir,
                &[&file_name],
            ));

            // Add the title to the vector.
            titles.push(sanitized.to_string());

            // Increment the index counter.
            index += 1;
        }

        logger::log(
            &format!(
                "{} file names are present in the output file name list",
                output_paths.len()
            ),
            false,
        );

        // We must now check that the number of files in the input
        // directory is equal to the number of entries from the output file list.
        if input_paths.len() != output_paths.len() {
            logger::log(
                &format!("The number of files in the input directory {} is not equal to the number of files in the output file list {}", input_paths.len(), output_paths.len()),
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

        logger::log(&"-".repeat(logger::SPLITTER), false);
        logger::log(
            &format!("Setup complete, in {} seconds.", now.elapsed().as_secs()),
            false,
        );

        logger::section("File Processing", true);

        // Process each media file.
        for (i, m) in &mut media.iter_mut().enumerate() {
            logger::subsection(&format!("File {} of {}", i + 1, media_len), true);

            let now = Instant::now();
            if !m.process(&self.output_paths[i], &self.titles[i], params) {
                break;
            }

            logger::log(
                &format!(
                    "Processing complete, in {} seconds.",
                    now.elapsed().as_secs()
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
        logger::log("All files have been successfully processed!", true);

        // Shutdown the computer after processing, if required.
        if params.misc_params.shutdown_upon_completion {
            match shutdown() {
                Ok(_) => logger::log("Attempting to shutdown down the computer...", true),
                Err(e) => logger::log(&format!("Failed to shutdown the computer: {}", e), true),
            }
        }
    }
}
