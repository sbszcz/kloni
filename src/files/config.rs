use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use super::{
    get_or_create_config_file,
    ConfigFileStatus::{Created, Existing},
};

pub const CONFIG_DEFAULT: &str = r#"
[[providers]]
provider = "github"
base_url = "https://git.acme-enterprise.org"
token = "s3cr3t"
symbol = "GH"

[[providers]]
provider = "bitbucket"
base_url = "https://bitbucket.acme-enterprise.org"
token = "s3cr3t"
symbol = "BB"
"#;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub providers: Vec<Provider>,
}

#[derive(Deserialize, Debug)]
pub struct Provider {
    pub provider: Type,
    pub base_url: String,
    pub token: String,
    pub symbol: Option<String>,
}

#[derive(Deserialize, Debug)]
pub enum Type {
    github,
    bitbucket,
}

#[derive(Deserialize, Debug)]
pub struct GithubConf {
    pub base_url: String,
    pub token: String,
    pub icon: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BitbucketConf {
    pub base_url: String,
    pub token: String,
    pub icon: Option<String>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("It looks like kloni has been executed for the first time. Please provide the necessary repo provider config in '{0}'")]
    FirstRun(PathBuf),
}

impl Config {
    pub fn get(_custom_config: Option<PathBuf>) -> anyhow::Result<Config> {
        let conf_file_status = get_or_create_config_file(CONFIG_DEFAULT)?;

        match conf_file_status {
            Created(conf_path) => Err(ConfigError::FirstRun(conf_path).into()),
            Existing(conf_file) => {
                let config_toml = read_to_string(&conf_file)?;
                let config = toml::from_str::<Config>(&config_toml)?;

                Self::validate_config(&config, &conf_file)?;

                Ok(config)
            }
        }
    }

    fn validate_config(config: &Config, conf_file: &Path) -> anyhow::Result<()> {
        if config.providers.is_empty() {
            return Err(ConfigError::FirstRun(conf_file.to_path_buf()).into());
        }

        Ok(())
    }
}
