pub mod socket;
pub mod server;

use crate::server::{ Server };

use std::{ env };

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(addr) = args.get(1) else {
        eprintln!("help: run [ip:port]");
        return;
    };

    println!("starting slottet server on {addr}...");
    let server = Server::new(addr.to_string());
    server.run()
        .await
        .unwrap();
}

