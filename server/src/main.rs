mod server;

use std::{ env };
use common::Result;
use crate::server::{ Server };

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let Some(addr) = args.get(1) else {
        eprintln!("help: run [ip:port]");
        return Ok(());
    };

    println!("starting slottet server on {addr}...");
    let mut server = Server::connect(addr.to_string())?;
    server.run()
        .await
        .unwrap();

    Ok(())
}
