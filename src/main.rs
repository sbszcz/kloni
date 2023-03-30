mod files;

use std::{path::Path};

use git2::{RemoteCallbacks, Cred, build::RepoBuilder, FetchOptions};
use kloni::{git_url_provider_by_config, files::config::Config, run_selector_for_git_urls};

fn folder_name_for_url(url: &String) -> &str {
    let parts = url.split("/");
    let collection = parts.collect::<Vec<&str>>();
    let git_folder_name = collection.last().unwrap();

    return git_folder_name.strip_suffix(".git").unwrap();
}

fn clone_into_folder(git_url: &str, destination_folder: &str) -> anyhow::Result<()>{
    
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent("git")
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fo);
    builder.clone(git_url, Path::new(destination_folder))?;
    
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let conf = Config::get(None)?;

    let github = git_url_provider_by_config(&conf)?;
    let github_user_repos = github.collect_clone_urls()?;
    let selected_items = run_selector_for_git_urls(github_user_repos);

    for item in selected_items.iter() {
        let url = item.output().to_string();
        let folder_name = folder_name_for_url(&url);
       
        if Path::new(folder_name).is_dir() {            
            println!("Could not clone selection. Folder '{}' already exists.", folder_name);
        } else {            
            println!("Cloning {} into folder './{}'", &url, folder_name);
            clone_into_folder(&url, folder_name)?;            
        }
    }

    println!("Done!");

    Ok(())
}
