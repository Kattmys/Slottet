use common::utils::{ Counter, CounterType };
use common::map::Map;
use common::{
    Result,
    LATENCY,
    TICK_RATE,
    network::*,
    components::*,
    systems,
    columns,
    utils,
    sleep_until,
};

use crate::map;

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
const DEFAULT_MAX_INVENTORY_SIZE: u8 = 3;

pub struct Server {
    server: RenetServer,
    transport: NetcodeServerTransport,
    world: Arc<RwLock<hecs::World>>,
    clients: BiMap<ClientId, hecs::Entity>,
    map: Map,
    last_render_spot: HashMap<hecs::Entity, Vec2>,
}

const RENDER_DISTANCE: usize = 40;

impl Server {
    pub fn connect(addr: String) -> Result<Self> {
        let (server, transport) = create_server(addr)?;
        let world = Arc::new(RwLock::new(hecs::World::new()));
        let clients = BiMap::new();

        let map_filename = "maps/test-map.toml";
        let tile_matrix = map::load_map::<64, 64>(map_filename)
            .expect(&*format!("could not load {map_filename}"));
        // crate::map::print_map(&map);

        let mut map = Map {
            origin_pos: vec2(0., 0.),
            width: 64,
            height: 64,
            tiles: Vec::new()
        };
        for column in tile_matrix {
            map.tiles.extend(&column);
        }
        
        let last_render_spot = HashMap::new();

        Ok(Self {
            server,
            transport,
            world,
            clients,
            map,
            last_render_spot,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut send_column_counter = Counter::new(COLUMN_SEND_COUNTER);
        let mut column_counter = Counter::new(NUM_COLUMNS);

        let mut interval = tokio::time::interval(
            Duration::from_millis(1000 / TICK_RATE)
        );
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_tick = tokio::time::Instant::now();

        loop {
            let now = interval.tick().await;
            let delta = now.duration_since(last_tick);
            last_tick = now;

            {
                let mut world = self.world.write().await;
                systems::update_movement(&mut world, &self.map, delta);
            }

            if send_column_counter.step(1) == 0 {
                self.send_column_msg(column_counter.step(1)).await;
            }
            
            // self.send_map().await;

            self.handle_network(delta).await.expect("network error");
        }
    }

    async fn send_map(&mut self) -> ServerMessage {
        // eftersom den förra inte fungerade
        ServerMessage::MapPart {
            index: 0,
            tiles: self.map.tiles.clone().into(),
        }
    }

    // async fn send_map(&mut self) {
    //     let world = self.world.read().await;
    //     let entities: Vec<(hecs::Entity, Vec2)> = world
    //         .query::<(hecs::Entity, &Transform)>()
    //         .iter()
    //         .filter(|(entity, transform)| match self.last_render_spot.get(entity) {
    //             Some(last) => last.distance(transform.pos) > RENDER_DISTANCE as f32,
    //             None => true,
    //         })
    //         .map(|(entity, transform)| (entity, transform.pos))
    //         .collect();

    //     for (entity, pos) in entities {
    //         // TODO handle connection error
    //         let Some(client_id) = self.clients.get_by_right(&entity) else {
    //             continue;
    //         };
    //         let mut tile_messages = vec![];

    //         // let row_len = self.map.get(0)
    //         //     .and_then(|row| Some(row.len()))
    //         //     .unwrap_or(0);

    //         let tile_x = pos.x as usize;
    //         let tile_y = pos.y as usize;

    //         // TODO take origin_pos in beaktande
    //         let north = tile_y.saturating_sub(RENDER_DISTANCE / 2);
    //         let south = (tile_y + RENDER_DISTANCE / 2)
    //             .min(self.map.height);
    //         let west = tile_x.saturating_sub(RENDER_DISTANCE / 2);
    //         let east = (tile_x + RENDER_DISTANCE / 2)
    //             .min(self.map.width);

    //         for y in north..south {
    //             let tiles = self.map.tiles[
    //                 (west + y * self.map.width)..(east + y * self.map.width)
    //             ].to_vec().into_boxed_slice();

    //             tile_messages.push(ServerMessage::MapPart {
    //                 index: y * self.map.height + west,
    //                 tiles,
    //             });
    //         }

    //         self.last_render_spot.insert(entity, pos);

    //         self.server.send_message(
    //             *client_id,
    //             DefaultChannel::ReliableUnordered,
    //             serialize(tile_messages).unwrap()
    //         );
    //     }
    // }

    async fn send_column_msg(&mut self, index: CounterType) {
        debug_assert_eq!(NUM_COLUMNS, 5);
        let world = self.world.read().await;
        let columns = match index {
            0 => columns!(world, [Player]),
            1 => columns!(world, [Transform]),
            2 => columns!(world, [MovementState]),
            // TODO do not send everyone's inventory to everyone
            3 => columns!(world, [Inventory]),
            4 => columns!(world, [Health]),
            _ => unreachable!()
        };
        self.server.broadcast_message(
            DefaultChannel::Unreliable,
            serialize(ServerMessage::World(columns)).unwrap()
        );
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

            ClientMessage::Movement { ref movement } => {
                let position = match self.world.read().await.get::<&Transform>(*entity) {
                    Ok(transform) => transform.pos,
                    _ => unreachable!()
                };

                let timestamp = utils::epoch_time_millis();

                let packet = ServerMessage::Movement {
                    entity: *entity,
                    movement: movement.clone(),
                    timestamp,
                    position,
                };

                self.server.broadcast_message(
                    DefaultChannel::ReliableUnordered,
                    serialize(packet).unwrap(),
                );

                let world = self.world.clone();
                let movement = movement.clone();
                let entity = entity.clone();
                tokio::task::spawn(async move {
                    sleep_until!(tokio, timestamp + LATENCY);
                    let world = world.write().await;
                    if let Ok(mut entity_movement) =
                        world.get::<&mut MovementState>(entity)
                    {
                        *entity_movement = movement;
                    }
                });
            }
        }
    }

    async fn handle_connect(&mut self, client_id: ClientId) {
        let entity_id = {
            let pos = vec2(32.0, 32.0);
            let inventory = vec![Item::new(ItemType::Axe)];
            let mut world = self.world.write().await;
            world.spawn(PlayerBundle {
                player: Player,
                transform: Transform::new(pos),
                movement_state: MovementState {
                    dir: Direction::None,
                },
                inventory: Inventory {
                    slots: inventory,
                    selected: 0,
                    max_size: DEFAULT_MAX_INVENTORY_SIZE,
                },
                health: Health::default(),
            })
        };

        let map_msg = self.send_map().await;

        self.clients.insert(client_id, entity_id);
        self.server.send_message(
            client_id,
            DefaultChannel::ReliableUnordered,
            serialize(ServerMessage::Messages(vec![
                ServerMessage::InitialData {
                    entity: entity_id,
                }.into(),
                ServerMessage::MapData {
                    origin_pos: Vec2::ZERO,
                    world_width: self.map.width,
                    world_height: self.map.height,
                }.into(),
                ServerMessage::World(
                    columns!(self.world.read().await, [
                        Player,
                        Transform,
                        MovementState,
                        Inventory,
                        Health
                    ])
                ).into(),
                map_msg.into(),
            ])).unwrap()
        );
    }

    async fn handle_disconnect(&mut self, client_id: ClientId, reason: String) {
        if let  Some((_client_id, entity)) = self.clients.remove_by_left(&client_id) {
            println!("client {client_id}: disconnected, reason: {reason}");
            let mut world = self.world.write().await;
            world.despawn(entity).unwrap();
            self.server.broadcast_message(
                DefaultChannel::ReliableUnordered,
                serialize(
                    ServerMessage::ClientDisconnect { entity }
                ).unwrap()
            );
        }
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
                    match deserialize::<ClientMessage>(&bytes) {
                        Ok(message) => {
                            self.handle_msg(client_id, message).await;
                        }
                        Err(e) => eprintln!(
                            "client {client_id}: malformed packet: {e:?}"
                        ),
                    }
                }
            }
        }

        // if !messages.is_empty() {
        //     self.server.broadcast_message(
        //         DefaultChannel::Unreliable,
        //         serialize(messages).unwrap()
        //     );
        // }

        self.transport.send_packets(&mut self.server);

        Ok(())
    }
}
