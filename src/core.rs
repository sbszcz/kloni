use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use skim::AsAny;
use thiserror::Error;

use crate::files::{file_is_empty, get_or_create_cache_file};

#[derive(Debug)]
pub struct CloneUrl(pub String);

#[derive(Error, Debug)]
pub enum KloniError {
    #[error("Configured context is invalid. Allowed contexts are 'github' or 'bitbucket'")]
    InvalidContext,

    #[error("Cache file for '{0}' is missing. This is unexpected behaviour.")]
    MissingCacheFile(String),
}

pub trait HttpProvider {
    fn request_from_remote(&self) -> anyhow::Result<Vec<CloneUrl>>;
}

pub trait FileProvider {
    fn name(&self) -> &str;

    fn load_from_file(&self, cache_file: &File) -> anyhow::Result<Vec<CloneUrl>> {
        let buffered = BufReader::new(cache_file);

        let mut clone_urls = vec![];

        for line in buffered.lines() {
            clone_urls.push(CloneUrl(line.unwrap()))
        }

        Ok(clone_urls)
    }

    fn update_file(&self, clone_urls: &[CloneUrl], cache_file: &mut File) -> anyhow::Result<()> {
        let urls = clone_urls
            .iter()
            .map(|gurl| gurl.0.to_owned())
            .collect::<Vec<String>>()
            .join("\n");

        write!(cache_file, "{urls}")?;

        Ok(())
    }
}

pub trait CloneUrlProvider: FileProvider + HttpProvider {
    fn collect_clone_urls(&self) -> anyhow::Result<Vec<CloneUrl>> {
        let cache_file = &mut get_or_create_cache_file(self.name().to_string()).unwrap();

        let cache_file_is_empty = file_is_empty(cache_file);

        let clone_urls = match cache_file_is_empty {
            true => {
                // println!("cache file is empty");
                let clone_urls = self.request_from_remote()?;
                self.update_file(&clone_urls, cache_file)?;
                clone_urls
            }
            false => {
                let urls = self.load_from_file(cache_file)?;
                // println!("cache file contains {} urls", urls.len());
                urls
            }
        };

        Ok(clone_urls)
    }
}
