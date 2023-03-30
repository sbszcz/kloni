# kloni

Quick git repo cloning cli tool.

This tool is still work in progress and can be used on your own risk.

For now only Github Enterprise is supported. Bitbucket is already prepared but not completed.

# Installation

1. Install rust via [rustup](https://rustup.rs/).
1. Clone this repo
1. Build `kloni` using `cargo`
    ```bash
    cargo build --release
    ```
1. Copy the binary to a usual executables location.
    ```bash
    cp target/release/kloni ~/.local/bin
    ```

# Usage

1. Run `kloni` without arguments. The first execution will fail and ask you to provide Github Enterprise connection information (url, [personal access token](https://docs.github.com/de/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)) via the generated `~/.config/kloni/config.toml`.
1. `kloni` uses [skim](https://github.com/lotabout/skim) as fuzzy finder library. You can use the `tab` key to select multiple repos at once to clone them within one run.
1. `kloni` caches all repos found in `~/.cache/kloni/github` and won't issue further http requests as long as this file exists. For updating your repo list you have to delete this file manually.


# Todo

* Obviously add some tests
* Implement bitbucket support
* Improve command line interface
  * `--help` argument
  * `update` sub command
  * `context` sub command