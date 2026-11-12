use common::{
    Result,
    network::*,
    components::*,
    utils::{ Counter, CounterType },
    columns,
};
use renet::{ ClientId, RenetServer, DefaultChannel, ServerEvent };
use renet_netcode::NetcodeServerTransport;
use std::time::{ Duration, Instant };
use std::collections::HashMap;
use glam::vec2;

const TICK_RATE: u64 = 60;
const COLUMN_SEND_COUNTER: u64 = 20;

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

    pub async fn run(&mut self) -> Result<()> {
        let mut last_updated = Instant::now();
        let mut send_column_counter = Counter::new(COLUMN_SEND_COUNTER);
        let mut column_counter = Counter::new(NUM_COLUMNS);

        loop {
            let now = Instant::now();
            let delta = now - last_updated;
            last_updated = now;

            let mut messages = Vec::new();

            if send_column_counter.step(1) == 0 {
                messages.push(self.get_column_msg(column_counter.step(1)).await);
            }

            // TODO remove expect
            self.handle_network(delta, messages).await.expect("network error");

            tokio::time::sleep(Duration::from_millis(1000/TICK_RATE)).await;
        }
    }

    async fn get_column_msg(&self, index: CounterType) -> ServerMessage {
        debug_assert_eq!(NUM_COLUMNS, 2);
        let columns = match index {
            0 => columns!(self.world, [Player]),
            1 => columns!(self.world, [Transform]),
            _ => unreachable!()
        };
        ServerMessage::World(columns)
    }

    async fn handle_msg(&mut self, _client_id: ClientId, _msg: ClientMessage) {
        // match msg {
        //     ClientMessage::MoveDirection(vec) {
        //         if let Some(transform) = self.world.get_mut::<Transform>(client_id) {
        //             transform.rot = vec;
        //         }
        //     }
        // }
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        let pos = vec2(0.0, 0.0);
        let entity_id = self.world.spawn((Player, Transform::new(pos)));
        self.clients.insert(client_id, entity_id);
        self.server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            serialize(vec![
                ServerMessage::InitialData {
                    entity: entity_id,
                },
                ServerMessage::World(columns!(self.world, [Player, Transform])),
            ]).unwrap()
        );
    }

    async fn handle_network(&mut self, delta: Duration, messages: Vec<ServerMessage>) -> Result<()> {
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
                    // TODO remove unwraps
                    let entity = self.clients.remove(&client_id).unwrap();
                    self.world.despawn(entity).unwrap();
                }
            }
        }

        for client_id in self.server.clients_id() {
            for channel in RENET_CHANNELS {
                while let Some(bytes) = self.server.receive_message(
                    client_id,
                    *channel,
                ) {
                    match deserialize::<Vec<ClientMessage>>(&bytes) {
                        Ok(messages) => {
                            for msg in messages {
                                self.handle_msg(client_id, msg).await;
                            }
                        }
                        Err(e) => eprintln!(
                            "client {client_id}: malformed packet: {e:?}"
                        ),
                    }
                }
            }
        }

        if !messages.is_empty() {
            self.server.broadcast_message(
                DefaultChannel::Unreliable,
                serialize(messages).unwrap()
            );
        }

        self.transport.send_packets(&mut self.server);

        Ok(())
    }
}
