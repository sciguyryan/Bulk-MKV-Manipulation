use crate::paths;

use std::{path::Path, process::Command};

pub fn run_mkv_extract(fp: &str, arg_type: &str, args: &[String]) {
    let path = get_mkvtoolnix_exe("mkvextract.exe");

    // Run the MKV extract process.
    let output = Command::new(path)
        .arg(fp)
        .arg(arg_type)
        // TODO: add the arguments here.
        .spawn()
        .expect("failed to run MKVToolnix extract process");
}

pub fn get_mkvtoolnix_exe(exe: &str) -> String {
    Path::new(paths::MKVTOOLNIX_BASE)
        .join(exe)
        .to_string_lossy()
        .to_string()
}
