use anyhow::Result;
use delivery_station::http::new_server;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next();
    let addr = match args.next() {
        Some(addr) => addr,
        None => "config.yaml".to_string(),
    };
    std::env::set_var("CONFIG_FILE", addr);
    use delivery_station::constants::CONFIG;
    let addr = &CONFIG.http_listen_address;
    let http_server = new_server(addr)?;
    println!("Listening on {}", addr);
    if let Err(e) = http_server.serve().await {
        eprintln!("Server error: {}", e);
    }
    Ok(())
}
