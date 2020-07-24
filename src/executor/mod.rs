use std::{
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::{anyhow, bail, Result};
use blocking::unblock;
use http::{Request, Response};
use hyper::{Body, Error};
use serde::Deserialize;
use serde::Serialize;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    config::{Action, Config, Repository, Step},
    constants::{CONFIG, NOTIFIER},
    trigger::TriggeredInfo,
};

mod environment;
mod ssh;

impl Repository {
    pub async fn execute(&self, triggered_info: &TriggeredInfo) -> Result<()> {
        let mut action_result = Vec::new();
        let steps_name = triggered_info.steps_name.as_deref();
        for i in self
            .get_steps(steps_name)
            .ok_or_else(|| anyhow!("missing steps or steps name is invalid"))?
        {
            let result = i.execute(&CONFIG, &self, triggered_info).await?;
            let is_success = result.success();
            action_result.push(result);
            if !is_success {
                break;
            }
        }
        let status = (&action_result)
            .last()
            .map(|i| i.status)
            .unwrap_or_else(|| 0);
        let result = StepsResult {
            status,
            action_result,
        };
        NOTIFIER.notify(&self.name, result).await?;
        Ok(())
    }
}

pub struct StepsResult {
    status: i32,
    action_result: Vec<StepResult>,
}

impl StepsResult {
    pub fn success(&self) -> bool {
        self.status == 0
    }

    pub async fn save_to_file(self, config: &Config) -> Result<Vec<StepLog>> {
        let dir = crate::tmp_filename(16);
        let dir = Path::new(&dir);
        let mut step_log = Vec::new();
        for (index, result) in self.action_result.iter().enumerate() {
            let log = result
                .save_to_file(config, &dir.join(index.to_string()))
                .await?;
            step_log.push(log);
        }
        Ok(step_log)
    }
}

pub struct StepResult {
    status: i32,
    description: Option<String>,
    stdout: Option<Vec<u8>>,
    stderr: Option<Vec<u8>>,
}

impl StepResult {
    pub fn new(
        status: i32,
        description: Option<String>,
        stdout: Option<Vec<u8>>,
        stderr: Option<Vec<u8>>,
    ) -> StepResult {
        StepResult {
            status,
            description,
            stdout,
            stderr,
        }
    }

    fn success(&self) -> bool {
        self.status == 0
    }

    async fn save_to_file(&self, config: &Config, parent_dir: &Path) -> Result<StepLog> {
        let dir = Path::new(&config.work_dir)
            .join("cache")
            .join("logs")
            .join(parent_dir);
        if !dir.exists() {
            create_dir_all(&dir).await?;
        }

        async fn write_and_get_url(
            url: &PathBuf,
            parent_dir: &Path,
            dir: &PathBuf,
            number: &str,
            out: Option<&Vec<u8>>,
        ) -> Result<Option<String>> {
            Ok(match out {
                Some(out) if !out.is_empty() => {
                    let mut file = File::create(dir.join(number)).await?;
                    file.write_all(out).await?;
                    Some(format!(
                        "{}?id={}",
                        url.to_string_lossy(),
                        parent_dir.join(number).to_string_lossy()
                    ))
                }
                _ => None,
            })
        }
        let url = Path::new(&config.base_url).join("logs");
        let stdout_url =
            write_and_get_url(&url, parent_dir, &dir, "1", self.stdout.as_ref()).await?;
        let stderr_url =
            write_and_get_url(&url, parent_dir, &dir, "2", self.stderr.as_ref()).await?;
        let step_log = StepLog {
            description: self.description.clone(),
            stdout: stdout_url,
            stderr: stderr_url,
        };
        Ok(step_log)
    }
}

impl Step {
    async fn execute(
        &self,
        config: &Config,
        repository: &Repository,
        triggered_info: &TriggeredInfo,
    ) -> Result<StepResult> {
        use std::process::Command;

        let envs = self.environment(config, repository, triggered_info);
        match &self.host {
            Some(host) => {
                let host = config
                    .host
                    .get(host)
                    .ok_or_else(|| anyhow!("invalid host: {}", host))?
                    .clone();
                let _self = self.clone();
                let work_dir = config.work_dir.clone();
                unblock!(_self.ssh(&host, &work_dir))
            }
            None => {
                let (mut cmd, args) = match &self.action {
                    Action::Script { name } => {
                        let script_name =
                            self.get_script_fullname(&config.work_dir, name.get_name())?;
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
                let output = unblock!(cmd.output())?;
                let mut step_result: StepResult = output.into();
                step_result.description = self.description.clone();
                Ok(step_result)
            }
        }
    }

    fn get_script_fullname(&self, work_dir: &str, name: &str) -> Result<PathBuf> {
        let script_name = Path::new(work_dir);
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
            description: None,
            stdout: Some(output.stdout),
            stderr: Some(output.stderr),
        }
    }
}

#[derive(Serialize)]
pub struct StepLog {
    description: Option<String>,
    stdout: Option<String>,
    stderr: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OutputStaticArgs {
    id: String,
}

pub async fn logs_handler(req: Request<Body>) -> Result<Response<Body>, Error> {
    async fn inner(req: Request<Body>) -> Result<Response<Body>> {
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
    match inner(req).await {
        Ok(result) => Ok(result),
        Err(err) => Ok(Response::new(Body::from(err.to_string()))),
    }
}
