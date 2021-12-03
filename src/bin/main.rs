#[macro_use]
extern crate log;

use delivery_station::{
    constants::CONFIG,
    http::new_server,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let addr = &CONFIG.listen_address;
    let http_server = match new_server(addr) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    info!("listening on {}", addr);
    if let Err(e) = http_server.serve().await {
        error!("server error: {}", e);
    }
}
