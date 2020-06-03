use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::{anyhow, bail, Result};
use blocking::unblock;
use http::{Request, Response};
use hyper::{Body, Error};
use serde::Deserialize;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    config::{Action, Config, Repository, Step},
    constants::{CONFIG, DING_TALK},
    trigger::TriggeredInfo,
};

mod ssh;

impl Repository {
    pub async fn execute(&self, triggered_info: &TriggeredInfo) -> Result<()> {
        let mut status = 0;
        let mut action_result = Vec::new();
        let steps_name = triggered_info.steps_name.as_deref();
        for i in self
            .get_steps(steps_name)
            .ok_or_else(|| anyhow!("missing steps or steps name is invalid"))?
        {
            let result = i.execute(&CONFIG, &self, triggered_info).await?;
            if result.success() {
                action_result.push(result);
            } else {
                status = result.status;
                action_result.push(result);
                break;
            }
        }
        let result = StepsResult {
            status,
            action_result,
        };
        let status_info = if result.success() {
            "success"
        } else {
            "failure"
        };
        let logs: Vec<String> = result
            .save_to_file(&CONFIG)
            .await?
            .iter()
            .map(|(stdout, stderr)| format!("[stdout]({}), [stderr]({})", stdout, stderr))
            .collect();
        let logs = logs.join("\n1. ");
        let message = format!(
            "## deployment task \n\n**repository:** {} \n\n**status:** {} \n\n**logs:** \n\n 1. {}",
            &self.name, status_info, logs,
        );
        DING_TALK
            .markdown(&format!("auto deploy: {}", &self.name), &message, None)
            .await?;
        Ok(())
    }
}

pub struct StepsResult {
    status: i32,
    action_result: Vec<StepResult>,
}

impl StepsResult {
    fn success(&self) -> bool {
        self.status == 0
    }

    async fn save_to_file(self, config: &Config) -> Result<Vec<(String, String)>> {
        let dir = crate::tmp_filename(16);
        let dir = Path::new(&dir);
        let mut url_list = Vec::new();
        for (index, result) in self.action_result.iter().enumerate() {
            let url = result
                .save_to_file(config, &dir.join(index.to_string()))
                .await?;
            url_list.push(url);
        }
        Ok(url_list)
    }
}

pub struct StepResult {
    status: i32,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl StepResult {
    pub fn new(status: i32, stdout: Option<Vec<u8>>, stderr: Option<Vec<u8>>) -> StepResult {
        StepResult {
            status,
            stdout,
            stderr,
        }
    }

    fn success(&self) -> bool {
        self.status == 0
    }

    async fn save_to_file(&self, config: &Config, parent_dir: &Path) -> Result<(String, String)> {
        let dir = Path::new(&config.work_dir)
            .join("cache")
            .join("logs")
            .join(parent_dir);
        if !dir.exists() {
            create_dir_all(&dir).await?;
        }

        if let Some(stdout) = &self.stdout {
            let mut file = File::create(dir.join("1")).await?;
            file.write_all(&stdout).await?;
        }
        if let Some(stderr) = &self.stderr {
            let mut file = File::create(dir.join("2")).await?;
            file.write_all(&stderr).await?;
        }
        let url = Path::new(&config.base_url).join("logs");
        Ok((
            format!(
                "{}?id={}",
                url.to_string_lossy(),
                parent_dir.join("1").to_string_lossy()
            ),
            format!(
                "{}?id={}",
                url.to_string_lossy(),
                parent_dir.join("2").to_string_lossy()
            ),
        ))
    }
}

impl Step {
    fn environment(
        &self,
        config: &Config,
        repository: &Repository,
        triggered_info: &TriggeredInfo,
    ) -> HashMap<String, String> {
        let mut environment = match &config.environment {
            Some(envs) => envs.clone(),
            None => HashMap::new(),
        };

        if let Some(envs) = &repository.environment {
            environment.extend(envs.clone());
        }

        if let Some(envs) = &self.environment {
            environment.extend(envs.clone());
        }

        environment.insert(
            "TRIGGERED_INFO_REPOSITORY".to_string(),
            triggered_info.repository.to_string(),
        );
        if let Some(branch) = &triggered_info.branch {
            environment.insert("TRIGGERED_INFO_BRANCH".to_string(), branch.to_string());
        }
        if let Some(tag) = &triggered_info.tag {
            environment.insert("TRIGGERED_INFO_TAG".to_string(), tag.to_string());
        }
        if let Some(steps_name) = &triggered_info.steps_name {
            environment.insert(
                "TRIGGERED_INFO_STEPS_NAME".to_string(),
                steps_name.to_string(),
            );
        }
        environment
    }

    async fn execute(
        &self,
        config: &Config,
        repository: &Repository,
        triggered_info: &TriggeredInfo,
    ) -> Result<StepResult> {
        let _self = self.clone();
        let config = config.clone();
        let repository = repository.clone();
        let triggered_info = triggered_info.clone();
        unblock!(_self.sync_execute(&config, &repository, &triggered_info))
    }

    fn sync_execute(
        &self,
        config: &Config,
        repository: &Repository,
        triggered_info: &TriggeredInfo,
    ) -> Result<StepResult> {
        use std::process::Command;

        let envs = self.environment(config, repository, triggered_info);
        match &self.host {
            Some(host) => self.ssh(host, config),
            None => {
                let (mut cmd, args) = match &self.action {
                    Action::Script { name } => {
                        let script_name = self.get_script_fullname(config, name.get_name())?;
                        let mut cmd = Command::new("sh");
                        cmd.arg(script_name);
                        (cmd, name.get_args())
                    }
                    Action::Command { command } => {
                        let cmd = Command::new(command.get_name());
                        (cmd, command.get_args())
                    }
                };
                if let Some(args) = args {
                    cmd.args(args);
                }
                if let Some(current_dir) = &self.current_dir {
                    cmd.current_dir(current_dir);
                }
                cmd.envs(envs);
                let output = cmd.output()?;
                Ok(output.into())
            }
        }
    }

    fn get_script_fullname(&self, config: &Config, name: &str) -> Result<PathBuf> {
        let script_name = Path::new(&config.work_dir);
        let script_name = if script_name.is_relative() {
            std::env::current_dir()?.join(script_name)
        } else {
            script_name.to_path_buf()
        };
        let script_name = script_name.join("scripts").join(name);
        if !script_name.exists() {
            bail!(
                r#"script "{}" does not exist"#,
                script_name.to_string_lossy()
            );
        }
        Ok(script_name)
    }
}

impl From<Output> for StepResult {
    fn from(output: Output) -> StepResult {
        StepResult {
            status: output.status.code().unwrap_or(0),
            stdout: Some(output.stdout),
            stderr: Some(output.stderr),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct OutputStaticArgs {
    id: String,
}

pub async fn logs_handler(req: Request<Body>) -> Result<Response<Body>, Error> {
    match logs_handler_inner(req).await {
        Ok(result) => Ok(result),
        Err(err) => Ok(Response::new(Body::from(err.to_string()))),
    }
}

async fn logs_handler_inner(req: Request<Body>) -> Result<Response<Body>> {
    let query = req
        .uri()
        .query()
        .ok_or_else(|| anyhow!("\"id\" is missing."))?;
    let query: OutputStaticArgs = serde_urlencoded::from_str(&query)?;
    let f = Path::new(&CONFIG.work_dir)
        .join("cache")
        .join("logs")
        .join(&query.id);
    let mut f = File::open(f).await?;
    let mut out = String::new();
    f.read_to_string(&mut out).await?;
    Ok(Response::new(Body::from(out)))
}
