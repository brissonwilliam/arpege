// TODO:
// list all files in import dir
// process each file

use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};

use crate::config;

pub fn import_new_files() -> Result<(), std::io::Error> {
    let dir = "./data/import"; // todo: read from config or env

    let files = std::fs::read_dir(dir).map_err(|e| {
        let err_msg = e.to_string();
        log::error!("could not open import dir {dir}. Err: {err_msg}");
        return e;
    })?;

    for path in files {
        log::info!("Importing file {}", path.unwrap().path().display())

        // todo: check if file exists through db

        // process into segments with ffmpeg
    }

    return Ok(());
}

