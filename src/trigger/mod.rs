use anyhow::Result;
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
    pub async fn delivery(self, config: &'static Config) -> Result<()> {
        if let Some(repo) = config.repository.iter().find(|i| i.name == self.repository) {
            tokio::spawn(async move { repo.execute(&self).await });
        }
        Ok(())
    }
}
