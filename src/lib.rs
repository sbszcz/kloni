pub mod core;
pub mod files;
pub mod github;

use std::{io::Cursor, path::Path};

use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use skim::{
    prelude::{Event, SkimItemReader, SkimOptionsBuilder},
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
        .map(|skim_out| match skim_out.final_event {
            Event::EvActAccept(_) => skim_out.selected_items,
            Event::EvActAbort => vec![],
            _ => vec![],
        })
        .unwrap_or_else(Vec::new)
}

pub fn folder_name_for_url(url: &String) -> &str {
    let parts = url.split("/");
    let collection = parts.collect::<Vec<&str>>();
    let git_folder_name = collection.last().unwrap();

    return git_folder_name.strip_suffix(".git").unwrap();
}

pub fn clone_into_folder(git_url: &str, destination_folder: &str) -> anyhow::Result<()> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks
        .credentials(|_url, _username_from_url, _allowed_types| Cred::ssh_key_from_agent("git"));

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    builder.clone(git_url, Path::new(destination_folder))?;

    Ok(())
}
