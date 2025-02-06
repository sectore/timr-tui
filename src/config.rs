use crate::constants::APP_NAME;
use color_eyre::eyre::{eyre, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct Config {
    pub log_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn init() -> Result<Self> {
        // default logs dir
        let log_dir = get_default_state_dir()?.join("logs");
        fs::create_dir_all(&log_dir)?;

        // default data dir
        let data_dir = get_default_state_dir()?.join("data");
        fs::create_dir_all(&data_dir)?;

        Ok(Self { log_dir, data_dir })
    }
}

pub fn get_project_dir() -> Result<ProjectDirs> {
    let dirs = ProjectDirs::from("", "", APP_NAME)
        .ok_or_else(|| eyre!("Failed to get project directories"))?;

    Ok(dirs)
}

fn get_default_state_dir() -> Result<PathBuf> {
    let dirs = get_project_dir()?;
    let directory: PathBuf = dirs
        .state_dir()
        .unwrap_or_else(|| dirs.data_local_dir())
        .to_path_buf();

    Ok(directory)
}
