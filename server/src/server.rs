use common::{
    Result,
    LATENCY,
    network::*,
    components::*,
    utils::{ self, Counter, CounterType },
    systems,
    columns,
};

use renet::{ ClientId, RenetServer, DefaultChannel, ServerEvent };
use renet_netcode::NetcodeServerTransport;
use std::time::{ Duration, Instant };
use std::collections::HashMap;
use glam::vec2;

const TICK_RATE: f64 = 120.;
const COLUMN_SEND_COUNTER: u64 = 20;

pub struct Server {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: hecs::World,
    clients: HashMap<ClientId, hecs::Entity>,
    message_buffer: Vec<(hecs::Entity, ClientMessage)>,
}

impl Server {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = create_server(addr)?;
        let world = hecs::World::new();
        let clients = HashMap::new();
        let message_buffer = Vec::new();

        Ok(Self {
            server,
            transport,
            world,
            clients,
            message_buffer,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut send_column_counter = Counter::new(COLUMN_SEND_COUNTER);
        let mut column_counter = Counter::new(NUM_COLUMNS);

        let mut last_updated = Instant::now();
        #[allow(unused_assignments)]
        let mut delta = Duration::ZERO;

        loop {
            let timer = tokio::spawn(async {
                tokio::time::sleep(Duration::from_secs_f64(1.0/TICK_RATE)).await;
            });
            let now = Instant::now();
            delta = now - last_updated;
            last_updated = now;

            let mut messages = Vec::new();

            if send_column_counter.step(1) == 0 {
                messages.push(self.get_column_msg(column_counter.step(1)).await);
            }

            // TODO remove expect
            self.handle_network(delta, messages).await.expect("network error");
            self.iterate_msg_buffer().await;
            systems::update_movement(&mut self.world, delta).await;

            // tokio::time::sleep(Duration::from_secs_f64(1.0/TICK_RATE).saturating_sub(delta)).await;
            let _ = timer.await;
        }
    }

    async fn get_column_msg(&self, index: CounterType) -> ServerMessage {
        debug_assert_eq!(NUM_COLUMNS, 3);
        let columns = match index {
            0 => columns!(self.world, [Player]),
            1 => columns!(self.world, [Transform]),
            2 => columns!(self.world, [MovementState]),
            _ => unreachable!()
        };
        ServerMessage::World(columns)
    }

    async fn iterate_msg_buffer(&mut self) {
        let now = utils::epoch_time();
        for (i, (entity, msg)) in self.message_buffer.iter().enumerate() {
            match msg {
                ClientMessage::Movement { movement, timestamp } => {
                    let dt = now - timestamp;
                    if dt >= LATENCY {
                        println!("{dt}");
                        if let Ok(mut entity_movement) = self.world.get::<&mut MovementState>(*entity) {
                            *entity_movement = movement.clone();
                        }
                        self.message_buffer.swap_remove(i);
                        break
                    }
                }
            }
        }
    }

    async fn handle_msg(&mut self, client_id: ClientId, msg: ClientMessage) {
        let Some(entity) = self.clients.get(&client_id) else {
            eprintln!("tried to handle message from invalid client {client_id}");
            return
        };
        match msg {
            ClientMessage::Movement { ref movement, timestamp } => {
                let packet = ServerMessage::Movement {
                    entity: *entity,
                    movement: movement.clone(),
                    timestamp,
                };

                self.server.broadcast_message_except(
                    client_id,
                    DefaultChannel::ReliableUnordered,
                    serialize(vec![packet]).unwrap(),
                );

                self.message_buffer.push((*entity, msg));
            }
        }
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        let pos = vec2(0.0, 0.0);
        let entity_id = self.world.spawn(PlayerBundle {
            player: Player,
            transform: Transform::new(pos),
            movement_state: MovementState {
                dir: Direction::None,
            },
        });

        self.clients.insert(client_id, entity_id);
        self.server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            serialize(vec![
                ServerMessage::InitialData {
                    entity: entity_id,
                },
                ServerMessage::World(
                    columns!(self.world, [
                        Player,
                        Transform,
                        MovementState
                    ])
                ),
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
