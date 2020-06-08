use once_cell::sync::Lazy;

use crate::{config::Config, http::Client, notifier::dingtalk::DingTalk};

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env().unwrap());
pub static CLIENT: Lazy<Client> = Lazy::new(|| Client::default());
pub static DING_TALK: Lazy<DingTalk<'static>> = Lazy::new(|| {
    DingTalk::new(
        &CLIENT,
        &CONFIG.dingtalk_access_token,
        &CONFIG.dingtalk_secret,
    )
});
