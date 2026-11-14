use common::{
    Result,
    LATENCY,
    TICK_RATE,
    network::*,
    components::*,
    utils::{ Counter, CounterType },
    systems,
    columns,
    sleep_until,
};

use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use renet::{ ClientId, RenetServer, DefaultChannel, ServerEvent };
use renet_netcode::NetcodeServerTransport;
use glam::vec2;

// TODO replace with tokio timer
const COLUMN_SEND_COUNTER: u64 = 20;

pub struct Server {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: Arc<Mutex<hecs::World>>,
    clients: HashMap<ClientId, hecs::Entity>,
}

impl Server {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = create_server(addr)?;
        let world = Arc::new(Mutex::new(hecs::World::new()));
        let clients = HashMap::new();

        Ok(Self {
            server,
            transport,
            world,
            clients,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut send_column_counter = Counter::new(COLUMN_SEND_COUNTER);
        let mut column_counter = Counter::new(NUM_COLUMNS);

        let delta_fix = Duration::from_secs_f32(1.0 / TICK_RATE);
        let mut interval = tokio::time::interval(delta_fix);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut dt_accumulator = Duration::ZERO;
        let mut last_tick = tokio::time::Instant::now();

        loop {
            let now = interval.tick().await;
            let dt = now.duration_since(last_tick);
            dt_accumulator += dt;
            last_tick = now;

            let mut messages = Vec::new();

            // TODO put in separate thread
            if send_column_counter.step(1) == 0 {
                messages.push(self.get_column_msg(column_counter.step(1)).await);
            }

            self.handle_network(delta_fix, messages).await.expect("network error");

            while delta_fix < dt_accumulator {
                // TODO remove expect
                let mut world = self.world.lock().await;
                systems::update_movement(&mut world, delta_fix).await;
                dt_accumulator -= delta_fix;
            }
        }
    }

    async fn get_column_msg(&self, index: CounterType) -> ServerMessage {
        debug_assert_eq!(NUM_COLUMNS, 3);
        let world = self.world.lock().await;
        let columns = match index {
            0 => columns!(world, [Player]),
            1 => columns!(world, [Transform]),
            2 => columns!(world, [MovementState]),
            _ => unreachable!()
        };
        ServerMessage::World(columns)
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

                let world = self.world.clone();
                let movement = movement.clone();
                let entity = entity.clone();
                tokio::task::spawn(async move {
                    sleep_until!(tokio, timestamp + LATENCY);
                    let world = world.lock().await;
                    if let Ok(mut entity_movement) = world.get::<&mut MovementState>(entity) {
                        *entity_movement = movement;
                    }
                });
            }
        }
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        let pos = vec2(0.0, 0.0);

        let entity_id = {
            let mut world = self.world.lock().await;
            world.spawn(PlayerBundle {
                player: Player,
                transform: Transform::new(pos),
                movement_state: MovementState {
                    dir: Direction::None,
                },
            })
        };

        self.clients.insert(client_id, entity_id);
        self.server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            serialize(vec![
                ServerMessage::InitialData {
                    entity: entity_id,
                },
                ServerMessage::World(
                    columns!(self.world.lock().await, [
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
                    let mut world = self.world.lock().await;
                    world.despawn(entity).unwrap();
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
