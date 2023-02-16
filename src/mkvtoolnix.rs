use crate::paths;

use std::{path::Path, process::Command};

pub fn get_mkvtoolnix_exe(exe: &str) -> String {
    Path::new(&paths::PATHS.mkvtoolnix)
        .join(exe)
        .to_string_lossy()
        .to_string()
}

pub fn run_mkv_extract(in_path: &str, out_path: &str, arg_type: &str, args: &[String]) -> i32 {
    let path = get_mkvtoolnix_exe("mkvextract.exe");

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
            2
        }
    } else {
        2
    }
}

pub fn run_mkv_merge(base_dir: &str, args: &[String]) -> i32 {
    let path = get_mkvtoolnix_exe("mkvmerge.exe");

    let r = Command::new(path).args(args).current_dir(base_dir).output();

    if let Ok(exit) = r {
        if let Some(code) = exit.status.code() {
            code
        } else {
            2
        }
    } else {
        2
    }
}
