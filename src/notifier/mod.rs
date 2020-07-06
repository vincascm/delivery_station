use anyhow::Result;

use crate::{constants::CONFIG, executor::StepsResult};

mod dingtalk;
pub use dingtalk::DingTalk;

const TPL: &str = include_str!("dingtalk.tpl");

pub struct Notifier<'a> {
    dingtalk: &'a DingTalk<'a>,
}

impl<'a> Notifier<'a> {
    pub fn new(dingtalk: &'a DingTalk<'a>) -> Notifier<'a> {
        Notifier { dingtalk }
    }

    pub async fn notify(&self, repository_name: &str, result: StepsResult) -> Result<()> {
        let status = result.success();
        let logs: Vec<_> = result
            .save_to_file(&CONFIG)
            .await?
            .into_iter()
            .map(|(stdout, stderr)| (stdout.unwrap_or_default(), stderr.unwrap_or_default()))
            .collect();
        let mut context = tera::Context::new();
        context.insert("repository_name", repository_name);
        context.insert("status", &status);
        context.insert("logs", &logs);
        let message = tera::Tera::one_off(&TPL, &context, false)?;
        self.dingtalk
            .markdown(&format!("auto deploy: {}", repository_name), &message, None)
            .await?;
        Ok(())
    }
}
