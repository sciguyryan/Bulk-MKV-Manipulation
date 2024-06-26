use crate::{logger, paths, script_file::ScriptFile};

use std::{path::Path, process::Command};

const FAIL_ERROR_CODE: i32 = 2;

pub fn get_exe(exe: &str) -> String {
    Path::new(&paths::PATHS.mkvtoolnix)
        .join(format!("{}.exe", exe))
        .display()
        .to_string()
}

/// Run the MKV extract process.
///
/// # Arguments
///
/// * `in_path` - The input file path.
/// * `out_path` - The output file path.
/// * `arg_type` - The type of action to be performed.
/// * `args` - A list of arguments to be passed to the extractor.
/// * `file_id` - The ID of the media file.
pub fn run_extract(
    in_path: &str,
    out_path: &str,
    arg_type: &str,
    args: &[String],
    file_id: usize,
) -> i32 {
    let path = get_exe("mkvextract");

    let temp_file = ScriptFile::new(file_id, "mkvextract");
    temp_file.write_string(format!(
        "\"{path}\" \"{in_path}\" {arg_type} {}",
        args.join(" ")
    ));

    let output = Command::new(temp_file.get_path())
        .current_dir(format!("{out_path}/{arg_type}"))
        .output();

    let result = match &output {
        Ok(o) => {
            if let Some(code) = o.status.code() {
                code
            } else {
                FAIL_ERROR_CODE
            }
        }
        Err(_) => FAIL_ERROR_CODE,
    };

    if result == FAIL_ERROR_CODE {
        logger::log(
            "MKV Extract was not successfully executed and yielded the following output:",
            false,
        );
        let out = output.unwrap();
        logger::log_output_lines(&String::from_utf8_lossy(&out.stderr), false);
    }

    result
}

/// Run the MKV merge process.
///
/// # Arguments
///
/// * `base_dir` - The base directory for the process.
/// * `args` - A list of arguments to be passed to the extractor.
/// * `file_id` - The ID of the media file.
pub fn run_merge(base_dir: &str, args: &[String], file_id: usize) -> i32 {
    let path = get_exe("mkvmerge");

    let temp_file = ScriptFile::new(file_id, "mkvmerge");
    temp_file.write_string(format!("\"{path}\" {}", args.join(" ")));

    let output = Command::new(path).args(args).current_dir(base_dir).output();
    let result = match &output {
        Ok(o) => {
            if let Some(code) = o.status.code() {
                code
            } else {
                FAIL_ERROR_CODE
            }
        }
        Err(_) => FAIL_ERROR_CODE,
    };

    if result == FAIL_ERROR_CODE {
        logger::log(
            "MKV Merge was not successfully executed and yielded the following output:",
            false,
        );
        let out = output.unwrap();
        logger::log_output_lines(&String::from_utf8_lossy(&out.stderr), false);
    }

    result
}
