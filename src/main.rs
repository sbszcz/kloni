mod files;

use std::path::Path;

use kloni::{
    clone_into_folder, clone_url_provider_by_config, files::config::Config, folder_name_for_url,
    run_selector_for_git_urls,
};

fn main() -> anyhow::Result<()> {
    let conf = Config::get(None)?;
    let provider = clone_url_provider_by_config(&conf)?;
    let user_repos = provider.collect_clone_urls()?;
    let selected_items = run_selector_for_git_urls(user_repos);

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
