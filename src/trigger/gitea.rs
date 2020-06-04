use anyhow::{anyhow, bail, Result};
use hyper::{header::CONTENT_TYPE, Body, Error, Request, Response};
use serde::Deserialize;

use super::TriggeredInfo;
use crate::constants::CONFIG;

pub async fn gitea_trigger(req: Request<Body>) -> Result<Response<Body>, Error> {
    match gitea_trigger_inner(req).await {
        Ok(r) => Ok(r),
        Err(e) => Ok(Response::new(Body::from(e.to_string()))),
    }
}

async fn gitea_trigger_inner(req: Request<Body>) -> Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    match parts.headers.get(CONTENT_TYPE) {
        Some(c) => {
            if c != "application/json" {
                bail!("invalid content-type");
            }
        }
        None => bail!("missing content-type"),
    }
    let header_signature = parts
        .headers
        .get("X-Gitea-Signature")
        .ok_or_else(|| anyhow!("missing signature"))?;
    let body = hyper::body::to_bytes(body).await?;
    let payload_signature = signature(&CONFIG.gitea_trigger_secret, &body)?;
    if header_signature != payload_signature.as_bytes() {
        bail!("signature error");
    }
    let body: GiteaForm = serde_json::from_slice(&body)?;
    let info: TriggeredInfo = body.into();
    info.delivery(&CONFIG).await?;
    Ok(Response::new(Body::from("success")))
}

fn signature(key: &str, payload: &[u8]) -> Result<String> {
    use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};

    let key = PKey::hmac(key.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.update(payload)?;
    let hmac = signer.sign_to_vec()?;
    Ok(hex::encode(hmac))
}

#[derive(Debug, Clone, Deserialize)]
pub struct GiteaForm {
    secret: String,
    #[serde(rename = "ref")]
    _ref: String,
    before: String,
    after: String,
    compare_url: String,
    commits: Vec<Commit>,
    head_commit: Option<String>,
    repository: Repository,
    pusher: User,
    sender: User,
}

impl Into<TriggeredInfo> for GiteaForm {
    fn into(self) -> TriggeredInfo {
        TriggeredInfo {
            repository: self.repository.full_name,
            branch: Some(self._ref),
            tag: None,
            steps_name: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    id: String,
    message: String,
    url: String,
    author: Author,
    committer: Author,
    verification: Option<String>,
    timestamp: String,
    added: Option<Vec<String>>,
    removed: Option<Vec<String>>,
    modified: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    id: i32,
    owner: User,
    name: String,
    pub full_name: String,
    description: String,
    empty: bool,
    private: bool,
    fork: bool,
    template: bool,
    parent: Option<String>,
    mirror: bool,
    size: i32,
    html_url: String,
    ssh_url: String,
    clone_url: String,
    original_url: String,
    website: String,
    stars_count: i32,
    forks_count: i32,
    watchers_count: i32,
    open_issues_count: i32,
    open_pr_counter: i32,
    release_counter: i32,
    default_branch: String,
    archived: bool,
    created_at: String,
    updated_at: String,
    permissions: Permissions,
    has_issues: bool,
    internal_tracker: InternalTracker,
    has_wiki: bool,
    has_pull_requests: bool,
    ignore_whitespace_conflicts: bool,
    allow_merge_commits: bool,
    allow_rebase: bool,
    allow_rebase_explicit: bool,
    allow_squash_merge: bool,
    avatar_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    id: i32,
    login: String,
    full_name: String,
    email: String,
    avatar_url: String,
    language: String,
    is_admin: bool,
    last_login: String,
    created: String,
    username: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    name: String,
    email: String,
    username: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Permissions {
    admin: bool,
    push: bool,
    pull: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InternalTracker {
    enable_time_tracker: bool,
    allow_only_contributors_to_track_time: bool,
    enable_issue_dependencies: bool,
}
