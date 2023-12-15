mod files;

use std::path::Path;

use kloni::{
    clone_into_folder, clone_url_provider_by_config, core::CloneUrl, files::config::Config,
    folder_name_for_url, run_selector_for_git_urls,
};

fn main() -> anyhow::Result<()> {
    let conf = Config::get(None)?;
    let providers = clone_url_provider_by_config(&conf)?;

    let mut selectable_repos: Vec<CloneUrl> = vec![];
    for provider in providers {
        println!("Collecting repo clone urls for '{}'", provider.name());
        let clone_urls = provider.collect_clone_urls()?;
        selectable_repos.extend(clone_urls)
    }

    let selected_items = run_selector_for_git_urls(selectable_repos);

    for item in selected_items.iter() {
        let url = item.output().to_string();
        let folder_name = folder_name_for_url(&url);

        if Path::new(folder_name).is_dir() {
            println!(
                "Could not clone selection. Folder '{}' already exists.",
                folder_name
            );
        } else {
            println!("Cloning {} into folder '{}'", &url, folder_name);
            clone_into_folder(&url, folder_name)?;
            println!("Done!");
        }
    }
    Ok(())
}
