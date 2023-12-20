pub mod config;

use anyhow::Context;
use directories::ProjectDirs;
use std::{
    fs::{create_dir, File, OpenOptions},
    io::Write,
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("no valid home directory path could be retrieved from the operating system")]
    InvalidHomeDir,
}

fn project_dirs() -> anyhow::Result<ProjectDirs> {
    let dirs = ProjectDirs::from("dev", "sbszcz", "kloni").ok_or(FsError::InvalidHomeDir)?;

    Ok(dirs)
}

enum ConfigFileStatus {
    Created(PathBuf),
    Existing(PathBuf),
}

pub fn get_or_create_config_file(default_content: &str) -> anyhow::Result<ConfigFileStatus> {
    let conf_dir_root = project_dirs()?.config_dir().to_owned();

    if !conf_dir_root.exists() {
        create_dir(&conf_dir_root).context("attempt to create '{conf_dir_root}' failed")?;
    }

    let config_toml_path = conf_dir_root.join("config.toml");

    match config_toml_path.exists() {
        true => Ok(ConfigFileStatus::Existing(config_toml_path)),
        false => {
            let mut conf_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true) // creates en empty file, if not exist
                .open(&config_toml_path)?;

            write!(conf_file, "{}", default_content)?;

            Ok(ConfigFileStatus::Created(config_toml_path))
        }
    }
}

pub fn get_or_create_cache_file(file_name: String) -> anyhow::Result<File> {
    let cache_dir_root = project_dirs()?.cache_dir().to_owned();

    if !cache_dir_root.exists() {
        create_dir(&cache_dir_root).context("could not create '{cache_dir_root}'")?;
    }

    let cache_file_path = cache_dir_root.join(file_name);

    match cache_file_path.exists() {
        true => {
            // println!(
            //     "opening existing cache file '{}'",
            //     cache_file_path.display()
            // );

            let cache_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&cache_file_path)
                .context(format!(
                    "clould not open cache '{}'",
                    cache_file_path.display()
                ))?;

            Ok(cache_file)
        }
        false => {
            // println!("creating cache file {}", cache_file_path.display());

            let cache_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true) // creates en empty file, if not exist
                .open(cache_file_path)
                .context("clould not open cache '{cache_file_path}'")?;
            Ok(cache_file)
        }
    }
}
pub fn file_is_empty(file: &File) -> bool {
    match file.metadata() {
        Ok(md) => md.len() == 0,
        _ => false,
    }
}
