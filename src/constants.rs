use lazy_static::lazy_static;

use crate::{config::Config, http::Client, notifier::dingtalk::DingTalk};

lazy_static! {
    pub static ref CONFIG: Config = Config::from_env().unwrap();
    pub static ref CLIENT: Client = Client::default();
    pub static ref DING_TALK: DingTalk<'static> = DingTalk::new(
        &CLIENT,
        &CONFIG.dingtalk_access_token,
        &CONFIG.dingtalk_secret
    );
}
