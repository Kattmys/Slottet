use protocol::{
    Result,
    network::*,
    components::*,
    columns,
};
use renet::{ ClientId, RenetServer, DefaultChannel, ServerEvent };
use renet_netcode::NetcodeServerTransport;
use std::time::{ Duration, Instant };
use std::collections::HashMap;
use glam::vec2;

pub struct Server {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: hecs::World,
    clients: HashMap<ClientId, hecs::Entity>,
}

impl Server {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = create_server(addr)?;
        let world = hecs::World::new();
        let clients = HashMap::new();
        Ok(Self { server, transport, world, clients })
    }

    async fn handle_msg(&mut self, msg: ClientMessage) {
        println!("{msg:?}");
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        use ServerMessage::*;
        let pos = vec2(0.0, 0.0);
        let entity_id = self.world.spawn((Player, Transform::new(pos)));
        self.clients.insert(client_id, entity_id);
        self.server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            serialize(vec![
                InitialData {
                    entity: entity_id,
                },
                World(columns!(self.world, [Player, Transform])),
            ]).unwrap()
        );
    }

    async fn handle_network(&mut self, delta: Duration) -> Result<()> {
        self.server.update(delta);
        self.transport.update(delta, &mut self.server)?;

        while let Some(event) = self.server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    println!("client {client_id}: connected");
                    self.handle_connect(client_id).await;
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    println!("client {client_id}: disconnected, reason: '{reason}'");
                    self.clients.remove(&client_id);
                }
            }
        }

        for client_id in self.server.clients_id() {
            for channel in RENET_CHANNELS {
                while let Some(bytes) = self.server.receive_message(
                    client_id,
                    *channel,
                ) {
                    match deserialize::<ClientMessages>(&bytes) {
                        Ok(messages) => {
                            for msg in messages {
                                self.handle_msg(msg).await;
                            }
                        }
                        Err(e) => eprintln!(
                            "client {client_id}: malformed packet: {e:?}"
                        ),
                    }
                }
            }
        }

        self.transport.send_packets(&mut self.server);

        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        let mut last_updated = Instant::now();

        loop {
            let now = Instant::now();
            let delta = now - last_updated;
            last_updated = now;

            // TODO remove expect
            self.handle_network(delta).await.expect("network error");

            tokio::time::sleep(Duration::from_millis(16)).await;
        }
    }
}
