pub mod bitbucket;
pub mod core;
pub mod files;
pub mod github;

use std::{io::Cursor, path::Path};

use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use skim::{
    prelude::{Event, SkimItemReader, SkimOptionsBuilder},
    Skim, SkimItem,
};

use crate::bitbucket::Bitbucket;
use crate::core::{CloneUrl, GitUrlProvider};
use crate::files::config::{Config, Type};
use crate::github::Github;

pub fn clone_url_provider_by_config(
    config: &Config,
) -> anyhow::Result<Vec<Box<dyn GitUrlProvider>>> {
    let results = config
        .providers
        .iter()
        .filter_map(|provider| -> Option<Box<dyn GitUrlProvider>> {
            let token = &provider.token;
            let symbol = &provider.symbol;

            match provider.provider {
                Type::github => {
                    let gh_base_url = format!("{}{}", &provider.base_url, github::USER_ORGS_PATH);
                    Some(Box::new(Github::new(
                        token.to_owned(),
                        gh_base_url,
                        symbol.to_owned(),
                    )))
                }

                Type::bitbucket => {
                    let bitbucket_base_url =
                        format!("{}{}", &provider.base_url, bitbucket::USER_PROJECTS_PATH);
                    Some(Box::new(Bitbucket::new(
                        token.to_owned(),
                        bitbucket_base_url,
                        symbol.to_owned(),
                    )))
                }
            }
        })
        .collect::<Vec<Box<dyn GitUrlProvider>>>();

    Ok(results)
}

pub fn run_selector_for_git_urls(clone_urls: Vec<CloneUrl>) -> Vec<std::sync::Arc<dyn SkimItem>> {
    let urls = clone_urls
        .iter()
        .map(|clone_url| {
            if !clone_url.1.is_empty() {
                format!("{} | {}", clone_url.1, clone_url.0.to_owned())
            } else {
                clone_url.0.to_owned().to_string()
            }
        })
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
        .unwrap_or_default()
}

pub fn folder_name_for_url(url: &str) -> &str {
    let parts = url.split('/');
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

pub fn remove_symbol_prefix(url: &str) -> &str {
    match url.find('|') {
        Some(idx) => &url[idx + 2..],
        None => url,
    }
}

#[cfg(test)]
mod tests {
    use crate::remove_symbol_prefix;

    #[test]
    pub fn should_successfully_remove_symbol_from_url() {
        assert_eq!(
            "git@git.acmecorp.com:organization/example.git",
            remove_symbol_prefix("󰊤 | git@git.acmecorp.com:organization/example.git")
        );

        assert_eq!(
            "git@git.acmecorp.com:organization/example.git",
            remove_symbol_prefix("  foo bar   | git@git.acmecorp.com:organization/example.git")
        );

        assert_eq!(
            "git@git.acmecorp.com:organization/example.git",
            remove_symbol_prefix("git@git.acmecorp.com:organization/example.git")
        )
    }
}
