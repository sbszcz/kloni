use serde::Deserialize;

use crate::core::{CloneUrl, FileProvider, GitUrlProvider, HttpProvider};

#[derive(Debug, Deserialize)]
pub struct ProjectList {
    pub size: i32,
    pub values: Vec<Project>,
    pub is_last_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: i32,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoList {
    pub size: i32,
    pub values: Vec<Repo>,
    pub is_last_page: bool,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub links: Vec<LinkList>,
}

#[derive(Debug, Deserialize)]
pub struct LinkList {
    pub clone: Vec<Link>,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: String,
    pub name: String,
}

pub struct Bitbucket {
    pub token: String,
    pub orgs_url: String,
}

pub const USER_ORGS_PATH: &str = "/rest/api/1.0/projects";

impl Bitbucket {
    pub fn new(token: String, orgs_url: String) -> Bitbucket {
        Bitbucket { token, orgs_url }
    }
}

impl HttpProvider for Bitbucket {
    fn request_from_remote(&self) -> anyhow::Result<Vec<CloneUrl>> {
        todo!()
    }
}

impl FileProvider for Bitbucket {
    fn name(&self) -> &str {
        "bitbucket"
    }
}

impl GitUrlProvider for Bitbucket {}
