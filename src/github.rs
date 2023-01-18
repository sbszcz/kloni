use attohttpc::{Method, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::core::{CloneUrl, CloneUrlProvider, FileProvider, HttpProvider};

#[derive(Debug, Deserialize)]
pub struct GithubRepo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub git_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Organization {
    pub repos_url: String,
}

#[derive(Debug)]
pub struct OrganizationRepoUrl(pub String);

pub struct Github {
    pub token: String,
    pub orgs_url: String,
}

#[derive(Error, Debug)]
pub enum HttpProblem {
    #[error("Invalid url '{0}'")]
    InvalidUrl(String),

    #[error("HTTP request to '{0}' failed: {1}")]
    RequestFailed(String, String),

    #[error("Can't deserialize response from '{0}': {1}")]
    DeserializationFailed(String, String),
}

impl Github {
    pub fn new(token: String, orgs_url: String) -> Github {
        Github { token, orgs_url }
    }

    pub fn fetch_clone_urls(&self) -> anyhow::Result<Vec<CloneUrl>> {
        let orgs = Github::get::<Vec<Organization>>(&self.token, &self.orgs_url)?;

        let repo_urls: Vec<OrganizationRepoUrl> = orgs
            .iter()
            .map(|org| OrganizationRepoUrl(org.repos_url.to_string()))
            .collect();

        let mut git_urls: Vec<CloneUrl> = vec![];

        for OrganizationRepoUrl(url) in repo_urls {
            let git_repos = Github::get::<Vec<GithubRepo>>(&self.token, &url)?;

            for GithubRepo {
                name: _,
                full_name: _,
                description: _,
                git_url,
            } in git_repos
            {
                git_urls.push(CloneUrl(git_url))
            }
        }

        Ok(git_urls)
    }

    pub fn get<R: DeserializeOwned>(token: &str, url: &str) -> anyhow::Result<R> {
        let response = RequestBuilder::try_new(Method::GET, url)
            .map_err(|_e| HttpProblem::InvalidUrl(url.to_string()))?
            .danger_accept_invalid_certs(true)
            .header("Accept", "application/vnd.github+json")
            .bearer_auth(token)
            .send()
            .map_err(|e| HttpProblem::RequestFailed(url.to_string(), e.to_string()))?;
        
        
        if !response.is_success() {
            return Err(HttpProblem::RequestFailed(url.to_string(), format!("status: {}", response.status())).into());
        }

        let deserialized_result =  response.json::<R>()
            .map_err(|e| HttpProblem::DeserializationFailed(url.to_string(), e.to_string()))?;

        Ok(deserialized_result)
    }
}

impl HttpProvider for Github {
    fn request_from_remote(&self) -> anyhow::Result<Vec<CloneUrl>> {
        let result = self.fetch_clone_urls()?;
        Ok(result)
    }
}

impl FileProvider for Github {
    fn name(&self) -> &str {
        "github"
    }
}

impl CloneUrlProvider for Github {}
