use std::{collections::HashMap, fs::File, slice::from_ref};

use anyhow::Result;
use serde::Deserialize;
use serde_yaml::{from_reader, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// delivery station http server listen address
    pub listen_address: String,
    /// SSH host
    pub host: HashMap<String, Host>,
    /// git repository list
    pub repository: Vec<Repository>,
    /// notifier list
    pub notifier: Option<Vec<Notifier>>,
    /// SSH environment
    pub environment: Option<HashMap<String, String>>,
    /// delivery station work directory, default is `/tmp`
    pub work_dir: Option<String>,
    /// delivery station http server url prefix
    pub base_url: Option<String>,
    /// extra config
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "snake_case", tag = "type", content = "config")]
pub enum Notifier {
    Dingtalk {
        access_token: String,
        secret: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Host {
    pub description: Option<String>,
    pub hostname: String,
    pub port: Option<u16>,
    pub user: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub name: String,
    pub description: Option<String>,
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
            Steps::Multiple(m) => m,
            Steps::Single(s) => from_ref(s),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Step {
    pub description: Option<String>,
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
    pub fn init() -> Result<Config> {
        use std::env::{args, var};

        let filename = args()
            .nth(1)
            .or_else(|| var("CONFIG_FILE").ok())
            .unwrap_or_else(|| "config.yaml".to_owned());
        let file = File::open(&filename)?;
        let config = from_reader(file)?;
        Ok(config)
    }
}
