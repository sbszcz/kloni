use attohttpc::{Method, RequestBuilder};
use serde::Deserialize;

use crate::core::{CloneUrl, FileProvider, GitUrlProvider, HttpProblem, HttpProvider};

#[derive(Debug, Deserialize)]
pub struct ProjectList {
    pub size: i32,
    pub values: Vec<Project>,

    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub links: LinkList,
}

#[derive(Debug, Deserialize)]
pub struct RepoList {
    pub size: i32,
    pub values: Vec<Repo>,
    #[serde(rename = "isLastPage")]
    pub is_last_page: bool,
    #[serde(rename = "nextPageStart")]
    pub next_page_start: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub id: i32,
    pub name: String,
    pub links: LinkList,
}

#[derive(Debug, Deserialize)]
pub struct LinkList {
    pub clone: Option<Vec<Link>>,
    #[serde(rename = "self")]
    pub slf: Option<Vec<Link>>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: String,
    pub name: Option<String>,
}

pub struct Bitbucket {
    pub token: String,
    pub projects_url: String,
    pub symbol: Option<String>,
}

pub const USER_PROJECTS_PATH: &str = "/rest/api/1.0/projects";

impl Bitbucket {
    pub fn new(token: String, projects_url: String, symbol: Option<String>) -> Bitbucket {
        Bitbucket {
            token,
            projects_url,
            symbol,
        }
    }

    pub fn fetch_clone_urls(&self, symbol: &str) -> anyhow::Result<Vec<CloneUrl>> {
        let project_list: Vec<Project> = Self::get_all_projects(&self.token, &self.projects_url)?;

        let mut git_urls = vec![];

        for Project {
            key,
            id: _,
            name: _,
            links: _,
        } in project_list.iter()
        {
            let mut project_url = self.projects_url.clone();
            project_url.push('/');
            project_url.push_str(key);

            let git_repos = Self::get_all_repos(&self.token, &project_url)?;

            for Repo {
                id: _,
                name: _,
                links,
            } in git_repos
            {
                if let Some(links) = links.clone {
                    let ssh_links: Vec<&Link> = links
                        .iter()
                        .filter(|link| match &link.name {
                            Some(name) => name == "ssh",
                            None => false,
                        })
                        .collect();

                    // println!("clone url {}", &ssh_links[0].href);
                    git_urls.push(CloneUrl(
                        ssh_links[0].href.to_owned(),
                        symbol.to_string(),
                    ))
                }
            }
        }

        // dbg!(&git_urls);

        Ok(git_urls)
    }

    pub fn get_all_projects(token: &str, url: &str) -> anyhow::Result<Vec<Project>> {
        let mut pages_remaining = true;
        let mut projects = vec![];
        let mut request_url = url.to_string();

        while pages_remaining {
            let response = RequestBuilder::try_new(Method::GET, &request_url)
                .map_err(|_e| HttpProblem::InvalidUrl(url.to_string()))?
                .danger_accept_invalid_certs(true)
                .bearer_auth(token)
                .send()
                .map_err(|e| HttpProblem::RequestFailed(url.to_string(), e.to_string()))?;

            if !response.is_success() {
                return Err(HttpProblem::RequestFailed(
                    url.to_string(),
                    format!("status: {}", response.status()),
                )
                .into());
            }

            let deserialized_result = response
                .json::<ProjectList>()
                .map_err(|e| HttpProblem::DeserializationFailed(url.to_string(), e.to_string()))?;

            // dbg!(&deserialized_result);

            pages_remaining = !deserialized_result.is_last_page;
            if pages_remaining && deserialized_result.next_page_start.is_some() {
                request_url = format!(
                    "{}?start={}",
                    url,
                    deserialized_result.next_page_start.unwrap()
                )
            }
            projects.extend(deserialized_result.values);
            // dbg!(&projects);
        }

        Ok(projects)
    }

    pub fn get_all_repos(token: &str, project_url: &str) -> anyhow::Result<Vec<Repo>> {
        let mut pages_remaining = true;
        let mut repos = vec![];
        let repos_base_url = project_url.to_string() + "/repos";
        let mut request_url = repos_base_url.clone();

        // println!("get all repos from {request_url}");
        while pages_remaining {
            let response = RequestBuilder::try_new(Method::GET, &request_url)
                .map_err(|_e| HttpProblem::InvalidUrl(request_url.clone()))?
                .danger_accept_invalid_certs(true)
                .bearer_auth(token)
                .send()
                .map_err(|e| HttpProblem::RequestFailed(project_url.to_string(), e.to_string()))?;

            if !response.is_success() {
                return Err(HttpProblem::RequestFailed(
                    request_url,
                    format!("status: {}", response.status()),
                )
                .into());
            }

            let deserialized_result = response.json::<RepoList>().map_err(|e| {
                HttpProblem::DeserializationFailed(project_url.to_string(), e.to_string())
            })?;

            // dbg!(&deserialized_result);

            pages_remaining = !deserialized_result.is_last_page;
            if pages_remaining && deserialized_result.next_page_start.is_some() {
                request_url = format!(
                    "{}?start={}",
                    &repos_base_url,
                    deserialized_result.next_page_start.unwrap()
                )
            }
            repos.extend(deserialized_result.values);
        }

        Ok(repos)
    }
}

impl HttpProvider for Bitbucket {
    fn request_from_remote(&self, symbol: &str) -> anyhow::Result<Vec<CloneUrl>> {
        let result = self.fetch_clone_urls(symbol)?;
        Ok(result)
    }
}

impl FileProvider for Bitbucket {
    fn name(&self) -> &str {
        "bitbucket"
    }
}

impl GitUrlProvider for Bitbucket {
    fn symbol(&self) -> String {
        self.symbol.to_owned().unwrap_or("".to_string())
    }
}
