
use anyhow::{anyhow, bail, Result};
use hyper::{header::CONTENT_TYPE, Body, Error, Request, Response};
use serde::Deserialize;

/// coding.net code push event of service hook
pub async fn trigger(req: Request<Body>) -> Result<Response<Body>, Error> {
    match inner_trigger(req).await {
        Ok(r) => Ok(r),
        Err(e) => Ok(Response::new(Body::from(e.to_string()))),
    }
}

async fn inner_trigger(req: Request<Body>) -> Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    //dbg!(&parts, &body);
    let x = hyper::body::to_bytes(body).await?;
    dbg!(&parts.headers);
    dbg!(&x);
    // "x-coding-signature"
    // TODO: parse coding web hook request body
    Ok(Response::new(Body::empty()))
}

fn signature(key: &str, payload: &[u8]) -> Result<String> {
    use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};

    let key = PKey::hmac(key.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.update(payload)?;
    let hmac = signer.sign_to_vec()?;
    Ok(hex::encode(hmac))
}

