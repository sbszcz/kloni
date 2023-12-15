mod files;

use std::path::Path;

use kloni::{
    clone_into_folder, clone_url_provider_by_config, core::CloneUrl, files::config::Config,
    folder_name_for_url, remove_symbol_prefix, run_selector_for_git_urls,
};

fn main() -> anyhow::Result<()> {
    let conf = Config::get(None)?;
    let providers = clone_url_provider_by_config(&conf)?;

    let mut symbols: Vec<String> = vec![];
    let mut selectable_repos: Vec<CloneUrl> = vec![];

    for provider in providers {
        symbols.push(provider.symbol());
        let clone_urls = provider.collect_clone_urls()?;
        selectable_repos.extend(clone_urls)
    }

    let selected_items = run_selector_for_git_urls(selectable_repos);

    for item in selected_items.iter() {
        let output = item.output().to_string();

        // probably a very hemdsaermiliche solution but it works
        let url = remove_symbol_prefix(&output);
        let folder_name = folder_name_for_url(url);

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
