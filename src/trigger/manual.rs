use anyhow::{bail, Result};
use hyper::{header::CONTENT_TYPE, Body, Error, Request, Response};

use super::TriggeredInfo;
use crate::constants::CONFIG;

pub async fn manual_trigger(req: Request<Body>) -> Result<Response<Body>, Error> {
    match manual_trigger_inner(req).await {
        Ok(r) => Ok(r),
        Err(e) => Ok(Response::new(Body::from(e.to_string()))),
    }
}

async fn manual_trigger_inner(req: Request<Body>) -> Result<Response<Body>> {
    if let Some(c) = req.headers().get(CONTENT_TYPE) {
        if c != "application/json" {
            bail!("invalid content-type");
        }
    } else {
        bail!("missing content-type");
    };
    let body = req.into_body();
    let body = hyper::body::to_bytes(body).await?;
    let body: TriggeredInfo = serde_json::from_slice(&body)?;
    let result = if body.delivery(&CONFIG).await? {
        "matched"
    } else {
        "skipped"
    };
    Ok(Response::new(Body::from(result)))
}
