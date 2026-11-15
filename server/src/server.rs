use common::utils::{ Counter, CounterType };
use common::terrain::Tile;
use common::{
    Result,
    LATENCY,
    TICK_RATE,
    network::*,
    components::*,
    systems,
    columns,
    sleep_until,
};

use crate::terrain::generate_terrain;

use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use renet::{ ClientId, RenetServer, DefaultChannel, ServerEvent };
use renet_netcode::NetcodeServerTransport;
use glam::{ vec2, Vec2 };
use bimap::BiMap;

// TODO replace with tokio timer
const COLUMN_SEND_COUNTER: u64 = 1;

pub struct Server {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: Arc<RwLock<hecs::World>>,
    clients: BiMap<ClientId, hecs::Entity>,
    terrain: [[Tile; 64]; 64],
    last_render_spot: HashMap<hecs::Entity, Vec2>,
}

const RENDER_DISTANCE: usize = 20;

impl Server {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = create_server(addr)?;
        let world = Arc::new(RwLock::new(hecs::World::new()));
        let clients = BiMap::new();

        let terrain = generate_terrain();
        let last_render_spot = HashMap::new();

        Ok(Self {
            server,
            transport,
            world,
            clients,
            terrain,
            last_render_spot,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut send_column_counter = Counter::new(COLUMN_SEND_COUNTER);
        let mut column_counter = Counter::new(NUM_COLUMNS);

        let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_RATE));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_tick = tokio::time::Instant::now();

        {
            let world = Arc::clone(&self.world);
            tokio::task::spawn(async move {
                Self::update_movement_loop(world).await;
            });
        }

        loop {
            let now = interval.tick().await;
            let delta = now.duration_since(last_tick);
            last_tick = now;

            let mut messages = Vec::new();

            if send_column_counter.step(1) == 0 {
                messages.push(self.get_column_msg(column_counter.step(1)).await);
            }
            
            self.send_terrain().await;

            self.handle_network(delta, messages).await.expect("network error");
        }
    }

    async fn send_terrain(&mut self) {
        let world = self.world.read().await;
        let entities: Vec<(hecs::Entity, Vec2)> = world
            .query::<(hecs::Entity, &Transform)>()
            .iter()
            .filter(|(entity, transform)| match self.last_render_spot.get(entity) {
                Some(last) => last.distance(transform.pos) > RENDER_DISTANCE as f32,
                None => true,
            })
            .map(|(entity, transform)| (entity, transform.pos))
            .collect();

        for (entity, pos) in entities {
            // TODO handle connection error
            let Some(client_id) = self.clients.get_by_right(&entity) else {
                continue;
            };
            let mut tile_messages = vec![];

            let row_len = self.terrain.get(0)
                .and_then(|row| Some(row.len()))
                .unwrap_or(0);

            let north = (pos.y as usize).saturating_sub(RENDER_DISTANCE / 2);
            let south = (pos.y as usize + RENDER_DISTANCE / 2).clamp(0, self.terrain.len());
            let west = (pos.x as usize).saturating_sub(RENDER_DISTANCE / 2);
            let east = (pos.x as usize + RENDER_DISTANCE / 2).clamp(0, row_len);

            for y in north..south {
                let mut row_index = y * row_len;
                let mut tiles: Vec<Tile> = vec![];
                let Some(row) = self.terrain.get(row_index) else {
                    continue;
                };

                for x in west..east {
                    match row.get(x) {
                        Some(tile) => tiles.push(*tile),
                        None       => row_index += 1,
                    }
                }

                tile_messages.push(ServerMessage::TerrainPart {
                    index: row_index,
                    tiles: tiles.into_boxed_slice(),
                });
            }

            self.last_render_spot.insert(entity, pos);

            // TODO make interface for sending messages to specific clients
            self.server.send_message(
                *client_id,
                DefaultChannel::ReliableUnordered,
                serialize(tile_messages).unwrap()
            );
        }
    }

    async fn update_movement_loop(world: Arc<RwLock<hecs::World>>) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / TICK_RATE));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_tick = tokio::time::Instant::now();

        loop {
            let now = interval.tick().await;
            let delta = now.duration_since(last_tick);
            last_tick = now;

            {
                let mut world = world.write().await;
                systems::update_movement(&mut world, delta);
            }
        }
    }

    async fn get_column_msg(&self, index: CounterType) -> ServerMessage {
        debug_assert_eq!(NUM_COLUMNS, 3);
        let world = self.world.read().await;
        let columns = match index {
            0 => columns!(world, [Player]),
            1 => columns!(world, [Transform]),
            2 => columns!(world, [MovementState]),
            _ => unreachable!()
        };
        ServerMessage::World(columns)
    }

    async fn handle_msg(&mut self, client_id: ClientId, msg: ClientMessage) {
        let Some(entity) = self.clients.get_by_left(&client_id) else {
            eprintln!("tried to handle message from invalid client {client_id}");
            return
        };

        match msg {
            ClientMessage::Disconnect => {
                self.handle_disconnect(client_id, "player left".to_string()).await;
                self.server.disconnect(client_id);
            }

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
                    let world = world.write().await;
                    if let Ok(mut entity_movement) = world.get::<&mut MovementState>(entity) {
                        *entity_movement = movement;
                    }
                });
            }
        }
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        let entity_id = {
            let mut world = self.world.write().await;
            world.spawn(PlayerBundle {
                player: Player,
                transform: Transform::new(vec2(0.0, 0.0)),
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
                ServerMessage::TerrainData {
                    origin_pos: Vec2::ZERO,
                    world_width: self.terrain.len(),
                    world_height: self.terrain.get(0)
                        .and_then(|row| Some(row.len()))
                        .unwrap_or(0),
                },
                ServerMessage::World(
                    columns!(self.world.read().await, [
                        Player,
                        Transform,
                        MovementState
                    ])
                ),
            ]).unwrap()
        );
    }

    async fn handle_disconnect(&mut self, client_id: ClientId, reason: String) {
        if let  Some((_client_id, entity)) = self.clients.remove_by_left(&client_id) {
            println!("client {client_id}: disconnected, reason: {reason}");
            let mut world = self.world.write().await;
            world.despawn(entity).unwrap();
            self.server.broadcast_message(
                DefaultChannel::ReliableUnordered,
                serialize(vec![
                    ServerMessage::ClientDisconnect { entity }
                ]).unwrap()
            );
        }
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
                    self.handle_disconnect(client_id, format!("{reason}")).await;
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
