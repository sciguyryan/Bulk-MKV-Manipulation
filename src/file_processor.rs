use serde_derive::{Deserialize, Serialize};

use crate::{conversion_params::unified::UnifiedParams, utils};

use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
};

#[derive(Deserialize, Serialize)]
#[allow(unused)]
pub enum PadType {
    Ten,
    Hundred,
    Thousand,
}

pub struct FileProcessor {
    pub input_paths: Vec<String>,
    pub output_paths: Vec<String>,
}

impl FileProcessor {
    pub fn new(
        in_dir: String,
        out_dir: String,
        out_names_path: String,
        out_name_start_from: usize,
        out_name_pad: PadType,
    ) -> Option<Self> {
        if !utils::dir_exists(&in_dir) {
            panic!("Input directory '{}' does not exist", in_dir);
        }

        if !utils::dir_exists(&out_dir) {
            panic!("Output directory '{}' does not exist", out_dir);
        }

        if !utils::file_exists(&out_names_path) {
            panic!("Output file names file '{}' does not exist", out_names_path);
        }

        let mut input_paths = Vec::new();
        let mut output_paths = Vec::new();

        // Read all of the files within the input directory.
        let paths = fs::read_dir(in_dir).unwrap();
        for path in paths.flatten() {
            if !path.path().is_file() {
                continue;
            }

            let p = format!("{}", path.path().display());

            // We currently only support the manipulation of MKV files.
            // All other file types will be ignored.
            if let Some(ext) = utils::get_file_extension(&p) {
                if ext != *"mkv" {
                    continue;
                }
            }

            input_paths.push(p);
        }

        // Read the file containing the output names.
        let mut index = out_name_start_from;
        let file = match File::open(out_names_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "An error occurred while attempting to open the output names file: {:?}",
                    e
                );
                return None;
            }
        };

        // Iterate over each line of the file.
        for line in BufReader::new(file).lines().flatten() {
            // Skip empty lines.
            if line.is_empty() {
                continue;
            }

            // Question marks need to be handled slightly differently
            // depending on context.
            // We also want to remove various other characters that are
            // invalid for use with NTFS filenames.
            let sanitized = utils::sanitize_ntfs_name(&line.replace("? ", " - "));

            // Handle the number padding.
            let file_name = match out_name_pad {
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

            // Add the fill output path to the vector.
            output_paths.push(utils::join_paths_to_string(&out_dir, &[&file_name]));

            // Increment the index counter.
            index += 1;
        }

        // We must now check that the number of files in the input
        // directory is equal to the number of entries from the
        // output file list.
        if input_paths.len() != output_paths.len() {
            eprintln!("The number of files in the input directory is not equal to the number of files in the output directory");
            return None;
        }

        Some(Self {
            input_paths,
            output_paths,
        })
    }

    /// Process each of the media files in the input directory.
    ///
    /// # Arguments
    ///
    /// * `props` - The [`MediaProcessParams`] to be used while processing the media files.
    ///
    pub fn process(&self, params: &UnifiedParams) {
        use crate::media_file::MediaFile;

        // Process the data from each of the media files.
        let media_len = self.input_paths.len();
        let mut media = Vec::with_capacity(media_len);
        for i in 0..media_len {
            if let Some(mf) = MediaFile::from_path(&self.input_paths[i]) {
                media.push(mf);
            }
        }

        // Process each media file.
        for (i, m) in &mut media.iter_mut().enumerate() {
            print!("Processing media file {} of {}...", i + 1, media_len);
            m.process(params, &self.output_paths[i]);
            print!(" Done!\r\n");

            // Delete the original file, if required.
            if params.remove_original_file {
                let _ = trash::delete(&self.input_paths[i]);
            }
        }
    }
}