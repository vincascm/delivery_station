use anyhow::Result;

use crate::{config::Notifier, executor::StepsResult};

mod dingtalk;

impl Notifier {
    pub async fn notify(
        &self,
        repository_name: &str,
        description: Option<&str>,
        result: &StepsResult,
    ) -> Result<()> {
        match self {
            Notifier::Dingtalk {
                access_token,
                secret,
            } => {
                let notifier = dingtalk::DingTalk::new(access_token, secret);
                notifier.notify(repository_name, description, result).await
            }
        }
    }
}
