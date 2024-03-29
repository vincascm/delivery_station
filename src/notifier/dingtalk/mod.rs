use anyhow::Result;
use chrono::Local;
use http::Request;
use hyper::Body;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};

use crate::{constants::CONFIG, executor::StepsResult, http::Client};

const URL: &str = "https://oapi.dingtalk.com/robot/send";
const TPL: &str = include_str!("dingtalk.tpl");

pub struct DingTalk<'a> {
    access_token: &'a str,
    secret: &'a str,
}

impl<'a> DingTalk<'a> {
    pub fn new(access_token: &'a str, secret: &'a str) -> DingTalk<'a> {
        DingTalk {
            access_token,
            secret,
        }
    }

    pub async fn notify(
        &self,
        repository_name: &str,
        description: Option<&str>,
        result: &StepsResult,
    ) -> Result<()> {
        let status = result.success();
        let logs = result.save_to_file(&CONFIG).await?;
        let mut context = tera::Context::new();
        context.insert("repository_name", repository_name);
        context.insert("repository_description", &description);
        context.insert("status", &status);
        context.insert("logs", &logs);
        let message = tera::Tera::one_off(TPL, &context, false)?;
        self.markdown(&format!("auto deploy: {}", repository_name), &message, None)
            .await?;
        Ok(())
    }

    fn sign(&self, timestamp: i64) -> Result<String> {
        use openssl::{base64::encode_block, hash::MessageDigest, pkey::PKey, sign::Signer};

        let key = PKey::hmac(self.secret.as_bytes())?;
        let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
        let payload = format!("{}\n{}", timestamp, self.secret);
        signer.update(payload.as_bytes())?;
        let hmac = signer.sign_to_vec()?;
        let sign = encode_block(&hmac);
        Ok(percent_encode(sign.as_bytes(), NON_ALPHANUMERIC).to_string())
    }

    async fn send(&self, message: Message) -> Result<Respond> {
        let timestamp = Local::now().timestamp_millis();
        let sign = self.sign(timestamp)?;
        let body: MessageInner = message.into();
        let body = serde_json::to_string(&body)?;
        let request = Request::builder()
            .uri(format!(
                "{}?access_token={}&timestamp={}&sign={}",
                URL, self.access_token, timestamp, sign
            ))
            .method("POST")
            .header("Content-Type", "application/json")
            .header("User-Agent", "hyper/0.1")
            .body(Body::from(body))?;
        let client = Client::default();
        let resp = client.request(request).await?;
        let body = hyper::body::to_bytes(resp).await?;
        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn markdown(&self, title: &str, text: &str, at: Option<&[&str]>) -> Result<Respond> {
        let at = at.map(|at| At {
            at_mobiles: at.iter().map(ToString::to_string).collect(),
            is_at_all: true,
        });
        let message = Message::Markdown {
            title: title.to_string(),
            text: text.to_string(),
            at,
        };
        self.send(message).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Respond {
    errcode: i32,
    errmsg: String,
}

#[derive(Debug, Clone, Serialize)]
struct MessageInner {
    msgtype: String,
    #[serde(flatten)]
    message: Message,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum Message {
    Markdown {
        title: String,
        text: String,
        at: Option<At>,
    },
}

impl From<Message> for MessageInner {
    fn from(message: Message) -> MessageInner {
        match message {
            Message::Markdown { .. } => MessageInner {
                msgtype: "markdown".to_string(),
                message,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename = "camelCase")]
struct At {
    at_mobiles: Vec<String>,
    is_at_all: bool,
}
