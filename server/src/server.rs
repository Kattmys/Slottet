use protocol::{ Result, ClientPacket, ServerPacket };
use renet::{ ClientId };
use crate::socket::{ Socket };
use std::time::{ Duration };

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut socket = Socket::connect(self.addr.clone())?;

        loop {
            if let Ok(packets) = socket.recv().await {
                let _responses = self.update(packets).await;
            };
            let _ = socket.broadcast(ServerPacket::Pong)
                .await
                .inspect_err(|e| eprintln!("Unable to broadcast pong: {e:?}"));

            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    }

    pub async fn update(&mut self, packets: Vec<(ClientId, ClientPacket)>) -> Vec<ServerPacket> {
        vec![] // TODO implement update game logic
    }
}
