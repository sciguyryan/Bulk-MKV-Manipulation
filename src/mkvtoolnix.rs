use crate::paths;

use std::{path::Path, process::Command};

pub fn run_mkv_extract(fp: &str, out_path: &str, arg_type: &str, args: &[String]) {
    let path = get_mkvtoolnix_exe("mkvextract.exe");

    let r = Command::new(path)
        .arg(fp)
        .arg(arg_type)
        .args(args)
        .current_dir(format!("{}/{}", out_path, arg_type))
        .output()
        .expect("failed to run MKVToolnix extract process");
}

pub fn get_mkvtoolnix_exe(exe: &str) -> String {
    Path::new(paths::MKVTOOLNIX_BASE)
        .join(exe)
        .to_string_lossy()
        .to_string()
}
