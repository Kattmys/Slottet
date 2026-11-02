use protocol::{ Result };
use protocol::{ ServerPacket, ClientPacket };
use protocol::network::{ setup, utils };

use renet::{ RenetServer, ServerEvent, DefaultChannel, ClientId };
use renet_netcode::{ NetcodeServerTransport };

use std::time::{ Instant };

pub struct Socket {
    server: RenetServer,
    transport: NetcodeServerTransport,

    last_updated: Instant,
}

impl Socket {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = setup::create_server(addr)?;
        let last_updated = Instant::now();
        Ok(Self { server, transport, last_updated })
    }

    pub async fn recv(&mut self) -> Result<Vec<(ClientId, ClientPacket)>> {
        let now = Instant::now();
        let delta = now - self.last_updated;
        self.last_updated = now;

        self.server.update(delta);
        self.transport.update(delta, &mut self.server)?;

        // TODO: handle connects...
        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("client (id: {client_id}) connected");
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("client (id: {client_id}) disconnected, reason: '{reason}'");
                }
            }
        }

        let mut packets = vec![];

        for client_id in self.server.clients_id() {
            while let Some(bytes) = self.server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
                // let text = String::from_utf8(message.into()).unwrap();
                match utils::deserialize::<ClientPacket>(&bytes) {
                    Ok(packet) => packets.push((client_id, packet)),
                    Err(e) => eprintln!("Malformed packet from client (id: {client_id}): {e:?}"),
                }
            }
        }

        Ok(packets)
    }

    pub async fn broadcast(&mut self, packet: ServerPacket) -> Result<()> {
        // for client_id in self.server.clients_id_iter().collect::<Vec<_>>().iter() {
        //     self.server.send_message(
        //         *client_id,
        //         DefaultChannel::Unreliable,
        //         utils::serialize(&packet)?
        //     );
        // }
        self.server.broadcast_message(
            DefaultChannel::ReliableOrdered,
            utils::serialize(&packet)?
        );

        self.transport.send_packets(&mut self.server);
        Ok(())
    }

    #[inline]
    pub fn send(&mut self, packet: ServerPacket, client_id: ClientId) -> Result<()> {
        self.server.send_message(
            client_id,
            DefaultChannel::Unreliable,
            utils::serialize(&packet)?
        );

        self.transport.send_packets(&mut self.server);
        Ok(())
    }
}

