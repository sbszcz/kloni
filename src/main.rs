mod files;

use kloni::{clone_url_provider_by_config, files::config::Config, run_selector_for_git_urls};

fn main() -> anyhow::Result<()> {
    let conf = Config::get(None)?;
    // dbg!(conf);

    let github = clone_url_provider_by_config(&conf)?;

    let github_user_repos = github.collect_clone_urls()?;

    let selected_items = run_selector_for_git_urls(github_user_repos);

    for item in selected_items.iter() {
        print!("{}", item.output());
    }

    Ok(())
}
