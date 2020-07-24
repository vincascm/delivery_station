use anyhow::Result;
use log::error;
use serde::Deserialize;

use crate::config::Config;

pub mod gitea;
pub mod manual;

#[derive(Debug, Clone, Deserialize)]
pub struct TriggeredInfo {
    pub repository: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub steps_name: Option<String>,
}

impl TriggeredInfo {
    pub async fn delivery(self, config: &'static Config) -> Result<bool> {
        if let Some(repo) = config.repository.iter().find(|i| i.name == self.repository) {
            if let Some(branch) = &repo.branch {
                if !match &self.branch {
                    Some(b) => branch == "@any" || b == branch,
                    None => false,
                } {
                    return Ok(false);
                }
            }
            if let Some(tag) = &repo.tag {
                if !match &self.tag {
                    Some(t) => tag == "@any" || t == tag,
                    None => false,
                } {
                    return Ok(false);
                }
            }
            tokio::spawn(async move {
                if let Err(e) = repo.execute(&self).await {
                    error!("delivery execute error: {}", e);
                }
            });
        }
        Ok(true)
    }
}
