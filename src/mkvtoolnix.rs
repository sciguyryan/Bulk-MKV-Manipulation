use crate::paths;

use std::{path::Path, process::Command};

pub fn get_mkvtoolnix_exe(exe: &str) -> String {
    Path::new(paths::MKVTOOLNIX_BASE)
        .join(exe)
        .to_string_lossy()
        .to_string()
}

pub fn run_mkv_extract(in_path: &str, out_path: &str, arg_type: &str, args: &[String]) {
    let path = get_mkvtoolnix_exe("mkvextract.exe");

    let r = Command::new(path)
        .arg(in_path)
        .arg(arg_type)
        .args(args)
        .current_dir(format!("{}/{}", out_path, arg_type))
        .output()
        .expect("failed to run MKVToolnix extract process");
}

pub fn run_mkv_merge(out_path: &str, args: &[String]) {
    let path = get_mkvtoolnix_exe("mkvmerge.exe");

    let r = Command::new(path)
        .args(args)
        .output()
        .expect("failed to run MKVToolnix merge process");
}
