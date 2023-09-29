use attohttpc::header::HeaderMap;
use attohttpc::{Method, RequestBuilder};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::core::{CloneUrl, FileProvider, GitUrlProvider, HttpProvider};

#[derive(Debug, Deserialize)]
pub struct GithubRepo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub ssh_url: String,
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

#[derive(Error, Debug, PartialEq)]
pub enum HttpProblem {
    #[error("Invalid url '{0}'")]
    InvalidUrl(String),

    #[error("HTTP request to '{0}' failed: {1}")]
    RequestFailed(String, String),

    #[error("Can't deserialize response from '{0}': {1}")]
    DeserializationFailed(String, String),
}

lazy_static! {
    static ref NEXT_LINK_REGEX: Regex = Regex::new(r#"(?i)(http\S*\d+)>;\s+(rel="next")"#).unwrap();
}

pub const USER_ORGS_PATH: &str = "/api/v3/user/orgs";

impl Github {
    pub fn new(token: String, orgs_url: String) -> Github {
        Github { token, orgs_url }
    }

    pub fn fetch_clone_urls(&self) -> anyhow::Result<Vec<CloneUrl>> {
        let orgs = Self::get_all_organizations(&self.token, &self.orgs_url)?;

        let repo_urls: Vec<OrganizationRepoUrl> = orgs
            .iter()
            .map(|org| OrganizationRepoUrl(org.repos_url.to_string()))
            .collect();

        let mut git_urls: Vec<CloneUrl> = vec![];

        for OrganizationRepoUrl(url) in repo_urls {
            let git_repos = Self::get_all_repos(&self.token, url.as_str())?;

            for GithubRepo {
                name: _,
                full_name: _,
                description: _,
                ssh_url,
            } in git_repos
            {
                // println!("{ssh_url}");
                git_urls.push(CloneUrl(ssh_url))
            }
        }

        Ok(git_urls)
    }

    pub fn get_all_organizations(token: &str, url: &str) -> anyhow::Result<Vec<Organization>> {
        let response = RequestBuilder::try_new(Method::GET, url)
            .map_err(|_e| HttpProblem::InvalidUrl(url.to_string()))?
            .danger_accept_invalid_certs(true)
            // .header("Accept", "application/vnd.github+json")
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
            .json::<Vec<Organization>>()
            .map_err(|e| HttpProblem::DeserializationFailed(url.to_string(), e.to_string()))?;

        Ok(deserialized_result)
    }

    pub fn get_all_repos(token: &str, url: &str) -> anyhow::Result<Vec<GithubRepo>> {
        let mut pages_remaining = true;
        let mut results = vec![];
        let mut request_url = url.to_string();

        while pages_remaining {
            // println!("calling: {request_url}");

            let response = RequestBuilder::try_new(Method::GET, &request_url)
                .map_err(|_e| HttpProblem::InvalidUrl(request_url.to_string()))?
                .danger_accept_invalid_certs(true)
                // .header("Accept", "application/vnd.github+json")
                .bearer_auth(token)
                .send()
                .map_err(|e| HttpProblem::RequestFailed(request_url.to_string(), e.to_string()))?;

            if !response.is_success() {
                return Err(HttpProblem::RequestFailed(
                    request_url,
                    format!("status: {}", response.status()),
                )
                .into());
            }

            pages_remaining = match Self::next_link(response.headers()) {
                Some(next_link) => {
                    request_url = next_link;
                    true
                }
                None => false,
            };

            // println!("pages remaining: {pages_remaining}");

            let deserialized_result = response.json::<Vec<GithubRepo>>().map_err(|e| {
                HttpProblem::DeserializationFailed(request_url.to_string(), e.to_string())
            })?;

            results.extend(deserialized_result);
        }

        Ok(results)
    }

    fn next_link(header_map: &HeaderMap) -> Option<String> {
        if let Some(header_value) = header_map.get("Link") {
            if let Ok(value) = header_value.to_str() {
                if let Some(captures) = NEXT_LINK_REGEX.captures(value) {
                    if let Some(link) = captures.get(1) {
                        return Some(link.as_str().to_string());
                    }
                }
            }
        }

        None
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

impl GitUrlProvider for Github {}

#[cfg(test)]
mod tests {

    use crate::github::Github;
    use attohttpc::header::HeaderMap;

    use crate::core::CloneUrl;
    use httpmock::prelude::*;
    use serde_json::json;

    #[test]
    pub fn should_successfully_extract_next_link_from_header() {
        let valid_link_header_value = r#"<https://acme.company.com/api/v3/organizations/12729/repos?per_page=20&page=2>; rel="next", <https://acme.company.com/api/v3/organizations/12729/repos?per_page=20&page=4>; rel="last""#;

        let mut hm = HeaderMap::new();
        hm.insert("Link", valid_link_header_value.parse().unwrap());

        let result = Github::next_link(&hm);

        assert_eq!(
            result,
            Some(
                "https://acme.company.com/api/v3/organizations/12729/repos?per_page=20&page=2"
                    .to_owned()
            )
        );
    }

    #[test]
    pub fn should_extract_nothing_if_link_header_is_missing() {
        let hm = HeaderMap::new();
        let result = Github::next_link(&hm);

        assert_eq!(result, None);
    }

    #[test]
    pub fn should_extract_nothing_if_regex_wont_match() {
        let invalid_link_header_value =
            r#"(?i)<htions/12729/repos?per_page=20&page=4>; rel="last""#;
        let mut hm = HeaderMap::new();
        hm.insert("Link", invalid_link_header_value.parse().unwrap());

        let result = Github::next_link(&hm);

        assert_eq!(result, None);
    }

    #[test]
    pub fn should_fetch_all_cloneable_ssh_urls() {
        let server = MockServer::start();
        let address = server.address().to_string();

        let user_orgs_path = "/api/v3/user/orgs";
        let user_orgs_url = format!("http://{address}{user_orgs_path}");

        let foo_org_repos_path = "/api/v3/orgs/FOO_ORG/repos";
        let foo_org_repos_url = format!("http://{address}{foo_org_repos_path}");

        let foo_org_next_link_path = "/api/v3/organizations/12345/repos";
        let foo_org_next_link_url = format!("http://{address}{foo_org_next_link_path}");

        // organization repos
        let user_organizations_mock = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(user_orgs_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    json!(
                        [
                          {
                            "repos_url": &foo_org_repos_url,
                          }
                        ]
                    )
                    .to_string(),
                );
        });

        // repo list for org page 1
        // Example Link: <https://localhost:port/api/v3/organizations/12345/repos?per_page=20&page=2>; rel="next", <https://acme.company.com/api/v3/organizations/12345/repos?per_page=20&page=4>; rel="last"

        let next_link = format!("<{foo_org_next_link_url}?per_page=20&page=2>; rel=\"next\", <{foo_org_next_link_url}?per_page=20&page=2>; rel=\"last\"");

        let organisation_repos_mock_page_1 = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(foo_org_repos_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .header("Link", next_link)
                .body(
                    json!(
                        [
                          {
                            "name": "fanzy-project",
                            "full_name": "FOO_ORG/fanzy-project",
                            "description": "A fanzy project",
                            "ssh_url": "git@localhost:FOO_ORG/fanzy-project.git"
                          }
                        ]
                    )
                    .to_string(),
                );
        });

        let organisation_repos_mock_page_2 = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(foo_org_next_link_path)
                .query_param("per_page", "20")
                .query_param("page", "2");

            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    json!(
                        [
                          {
                            "name": "fanzy-project-2",
                            "full_name": "FOO_ORG/fanzy-project-2",
                            "description": "A second fanzy project",
                            "ssh_url": "git@localhost:FOO_ORG/fanzy-project-2.git"
                          }
                        ]
                    )
                    .to_string(),
                );
        });

        let github = Github::new("s3cr3t".to_string(), user_orgs_url);
        let cloneable_urls = github.fetch_clone_urls().unwrap();

        user_organizations_mock.assert();
        organisation_repos_mock_page_1.assert();
        organisation_repos_mock_page_2.assert();

        assert_eq!(&cloneable_urls.len(), &2);
        assert_eq!(
            cloneable_urls.get(0),
            Some(&CloneUrl(
                "git@localhost:FOO_ORG/fanzy-project.git".to_string()
            ))
        );
        assert_eq!(
            cloneable_urls.get(1),
            Some(&CloneUrl(
                "git@localhost:FOO_ORG/fanzy-project-2.git".to_string()
            ))
        );
    }

    #[test]
    fn should_fail_when_org_repos_json_response_is_not_parsable() {
        let server = MockServer::start();
        let address = server.address().to_string();

        let user_orgs_path = "/api/v3/user/orgs";
        let user_orgs_url = format!("http://{address}{user_orgs_path}");

        let foo_org_repos_path = "/api/v3/orgs/FOO_ORG/repos";
        let foo_org_repos_url = format!("http://{address}{foo_org_repos_path}");

        // organization repos
        let user_organizations_mock = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(user_orgs_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    json!(
                        [
                          {
                            "repos_url": &foo_org_repos_url,
                          }
                        ]
                    )
                    .to_string(),
                );
        });

        let organisation_repos_mock = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(foo_org_repos_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body("{");
        });

        let github = Github::new("s3cr3t".to_string(), user_orgs_url);
        let result = github.fetch_clone_urls();

        user_organizations_mock.assert();
        organisation_repos_mock.assert();

        assert!(result.is_err());

        let expected_error_message = format!("Can't deserialize response from '{foo_org_repos_url}': Json Error: invalid type: map, expected a sequence at line 1 column 1");
        assert_eq!(format!("{}", result.unwrap_err()), expected_error_message);
    }

    #[test]
    fn should_fail_when_user_orgs_json_response_is_not_parsable() {
        let server = MockServer::start();
        let address = server.address().to_string();

        let user_orgs_path = "/api/v3/user/orgs";
        let user_orgs_url = format!("http://{address}{user_orgs_path}");

        let foo_org_repos_path = "/api/v3/orgs/FOO_ORG/repos";

        // organization repos
        let user_organizations_mock = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(user_orgs_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body("foo");
        });

        let organisation_repos_mock = server.mock(|when, then| {
            when.method("GET")
                .header("Authorization", "Bearer s3cr3t")
                .path(foo_org_repos_path);
            then.status(200)
                .header("content-type", "application/json; charset=utf-8")
                .body(
                    json!(
                        [
                          {
                            "name": "fanzy-project-2",
                            "full_name": "FOO_ORG/fanzy-project-2",
                            "description": "A second fanzy project",
                            "ssh_url": "git@localhost:FOO_ORG/fanzy-project-2.git"
                          }
                        ]
                    )
                    .to_string(),
                );
        });

        let github = Github::new("s3cr3t".to_string(), user_orgs_url.clone());
        let result = github.fetch_clone_urls();

        user_organizations_mock.assert();
        organisation_repos_mock.assert_hits(0);

        assert!(result.is_err());

        let expected_error_message = format!(
            "Can't deserialize response from '{}': Json Error: expected ident at line 1 column 2",
            user_orgs_url
        );
        assert_eq!(format!("{}", result.unwrap_err()), expected_error_message);
    }

    #[test]
    fn should_fail_when_url_is_invalid() {
        let github = Github::new("s3cr3t".to_string(), "bonkers".to_string());
        let result = github.fetch_clone_urls();

        assert!(result.is_err());

        assert_eq!(format!("{}", result.unwrap_err()), "Invalid url 'bonkers'");
    }
}
