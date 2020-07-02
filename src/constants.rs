use once_cell::sync::Lazy;

use crate::{config::Config, http::Client, notifier::dingtalk::DingTalk};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env().unwrap());
pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::default());
pub static DING_TALK: Lazy<DingTalk<'static>> = Lazy::new(|| {
    let dingtalk = &CONFIG
        .notifier
        .dingtalk
        .as_ref()
        .expect("missing dingtalk in config file");
    DingTalk::new(&CLIENT, &dingtalk.access_token, &dingtalk.secret)
});
