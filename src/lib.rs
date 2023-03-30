pub mod core;
pub mod files;
pub mod github;

use std::io::Cursor;

use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim, SkimItem,
};

use crate::core::{CloneUrl, GitUrlProvider, KloniError};
use crate::files::config::Config;
use crate::github::Github;

pub fn git_url_provider_by_config(config: &Config) -> anyhow::Result<Box<dyn GitUrlProvider>> {
    match &config.context {
        Some(context) if context == "github" => {
            // config struct has already been validated so we should be safe here (famous last words)
            let github_conf = config.github.as_ref().unwrap();

            let token = &github_conf.token;
            let orgs_url = format!("{}/api/v3/user/orgs", &github_conf.base_url);

            Ok(Box::new(Github::new(token.to_owned(), orgs_url)))
        }
        _ => Err(KloniError::InvalidContext.into()),
    }
}

pub fn run_selector_for_git_urls(clone_urls: Vec<CloneUrl>) -> Vec<std::sync::Arc<dyn SkimItem>> {
    let urls = clone_urls
        .iter()
        .map(|gurl| gurl.0.to_owned())
        .collect::<Vec<String>>()
        .join("\n");

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(true)
        .exact(true)
        .build()
        .unwrap();

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(urls));

    // `run_with` would read and show items from the stream
    Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new)
}
