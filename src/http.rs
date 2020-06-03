use std::{future::Future, net::ToSocketAddrs};

use anyhow::{anyhow, Result};
use http::{Request, Response};
use hyper::{
    client::HttpConnector,
    server::{conn::AddrIncoming, Builder as HyperBuilder},
    Body, Server as HyperServer,
};
use hyper_tls::HttpsConnector;
use routerify::{Router, RouterBuilder, RouterService};

use crate::{
    executor::logs_handler,
    trigger::{gitea::gitea_trigger, manual::manual_trigger},
};

pub struct Server {
    http: HyperBuilder<AddrIncoming>,
    router: RouterBuilder<Body, hyper::Error>,
}

impl Server {
    pub fn new<T: ToSocketAddrs>(addr: T) -> Result<Server> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("http listen address is required."))?;
        let http = HyperServer::bind(&addr).http1_only(true);
        let router = Router::builder();
        let server = Server { http, router };
        Ok(server)
    }

    pub fn get<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static,
    {
        let router = self.router.get(path, handler);
        Self {
            router,
            http: self.http,
        }
    }

    pub fn post<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Body>, hyper::Error>> + Send + 'static,
    {
        let router = self.router.post(path, handler);
        Self {
            router,
            http: self.http,
        }
    }

    pub async fn serve(self) -> Result<()> {
        let router = self.router.build()?;
        let service = RouterService::new(router)?;
        let server = self.http.serve(service);
        Ok(server.await?)
    }
}

pub fn new_server<T: ToSocketAddrs>(addr: T) -> Result<Server> {
    let server = Server::new(addr)?;
    let server = server
        .post("/gitea_trigger", gitea_trigger)
        .post("/manual_trigger", manual_trigger)
        .get("/logs", logs_handler);
    Ok(server)
}

pub struct Client(hyper::Client<HttpsConnector<HttpConnector>, Body>);

impl Client {
    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>> {
        let resp = self.0.request(req).await?;

        /*
        use std::io::Write;
        let body = hyper::body::to_bytes(resp).await?;
        let mut f = std::fs::File::create("x.html")?;
        f.write_all(&body)?;
        Err(anyhow!("xx"))
        */

        if resp.status().is_success() {
            Ok(resp)
        } else {
            Err(anyhow!(
                "remote server error, http status code: {}",
                resp.status().as_u16()
            ))
        }
    }
}

impl Default for Client {
    fn default() -> Client {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .http1_title_case_headers(true)
            .build(https);
        Client(client)
    }
}
