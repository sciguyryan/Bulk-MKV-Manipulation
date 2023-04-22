use crate::paths;

use std::{path::Path, process::Command};

const FAIL_ERROR_CODE: i32 = 2;

pub fn get_exe(exe: &str) -> String {
    Path::new(&paths::PATHS.mkvtoolnix)
        .join(format!("{}.exe", exe))
        .display()
        .to_string()
}

pub fn run_extract(in_path: &str, out_path: &str, arg_type: &str, args: &[String]) -> i32 {
    let path = get_exe("mkvextract");

    let r = Command::new(path)
        .arg(in_path)
        .arg(arg_type)
        .args(args)
        .current_dir(format!("{out_path}/{arg_type}"))
        .output();

    if let Ok(exit) = r {
        if let Some(code) = exit.status.code() {
            code
        } else {
            FAIL_ERROR_CODE
        }
    } else {
        FAIL_ERROR_CODE
    }
}

pub fn run_merge(base_dir: &str, args: &[String]) -> i32 {
    let path = get_exe("mkvmerge");

    let r = Command::new(path).args(args).current_dir(base_dir).output();

    if let Ok(exit) = r {
        if let Some(code) = exit.status.code() {
            code
        } else {
            FAIL_ERROR_CODE
        }
    } else {
        FAIL_ERROR_CODE
    }
}
