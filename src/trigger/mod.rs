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
    pub async fn delivery(&self, config: &Config) -> Result<()> {
        for i in &config.repository {
            if i.name == self.repository {
                i.execute(&self).await?;
            }
        }
        Ok(())
    }
}
