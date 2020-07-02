use std::{collections::HashMap, fs::File, slice::from_ref};

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub http_listen_address: String,
    pub gitea_trigger_secret: String,
    pub base_url: String,
    pub work_dir: String,
    pub notifier: Notifier,
    pub environment: Option<HashMap<String, String>>,
    pub host: HashMap<String, Host>,
    pub repository: Vec<Repository>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Notifier {
    pub dingtalk: Option<Dingtalk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Dingtalk {
    pub access_token: String,
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Host {
    pub hostname: String,
    pub port: Option<u16>,
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub name: String,
    pub environment: Option<HashMap<String, String>>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    steps: CompositeSteps,
}

impl Repository {
    pub fn get_steps(&self, steps_name: Option<&str>) -> Option<&[Step]> {
        match &self.steps {
            CompositeSteps::Multiple(m) => {
                let steps_name = steps_name.unwrap_or("default");
                m.get(steps_name).and_then(|s| s.get())
            }
            CompositeSteps::Single(s) => s.get(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CompositeSteps {
    Multiple(HashMap<String, Steps>),
    Single(Steps),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Steps {
    Multiple(Vec<Step>),
    Single(Step),
}

impl Steps {
    pub fn get(&self) -> Option<&[Step]> {
        Some(match self {
            Steps::Multiple(m) => &m,
            Steps::Single(s) => from_ref(s),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Step {
    pub host: Option<String>,
    pub current_dir: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    #[serde(flatten)]
    pub action: Action,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Action {
    Command { command: Command },
    Script { name: Command },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Command {
    Single(String),
    WithArgs(Vec<String>),
}

impl Command {
    pub fn get_name(&self) -> &str {
        match self {
            Command::Single(s) => s,
            Command::WithArgs(w) => &w[0],
        }
    }
    pub fn get_args(&self) -> Option<&[String]> {
        match self {
            Command::Single(_) => None,
            Command::WithArgs(w) => Some(&w[1..]),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Config> {
        let file = std::env::var("CONFIG_FILE")?;
        let file = File::open(&file)?;
        let config = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}
