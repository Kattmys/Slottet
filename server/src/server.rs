use protocol::{ Result, ClientPacket, ServerPacket };
use renet::{ ClientId };
use crate::socket::{ Socket };

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
                let responses = self.update(packets);
            };
            socket.broadcast(ServerPacket::Pong);
        }
    }

    pub async fn update(&mut self, packets: Vec<(ClientId, ClientPacket)>) -> Vec<ServerPacket> {
        vec![] // TODO implement update game logic
    }
}
