mod texture;
mod keybinds;
mod map;

use crate::texture::TextureManager;
use crate::keybinds::{ Event, KeyMap, load_keymap };
use crate::map::Map;
use common::map::{ Tile, FrontTile, BackTile, TILE_SIZE };
use common::{
    Result,
    LATENCY,
    TICK_RATE,
    sleep_until,
};
use common::network::{
    ClientMessage,
    ServerMessage,
    create_client,
    RENET_CHANNELS,
    serialize,
    deserialize,
};
use common::components::{ Transform, MovementState, Direction };
use common::{ systems, utils };

use macroquad::prelude::*;
use std::env;
use std::time::{ Duration, Instant };
use std::sync::{ Arc, RwLock };
use std::collections::HashMap;
use renet::{ RenetClient, DefaultChannel };
use renet_netcode::NetcodeClientTransport;

struct Client {
    client: RenetClient,
    transport: NetcodeClientTransport,
    world: Arc<RwLock<hecs::World>>,
    player: hecs::Entity,
    keymap: KeyMap,
    textures: TextureManager,
    position_offsets: HashMap<hecs::Entity, Vec2>,
    map: Map,
}

macro_rules! update_components {
    ($world:expr, $columns:expr, [$($component:ident),*]) => {{
        $(
            for (id, obj) in $columns.$component {
                $world.insert(id, (obj,)).unwrap();
            }
        )*
    }}
}

impl Client {
    async fn connect(addr: String) -> Result<Self> {
        let (client, transport) = create_client(addr)?;
        let world = Arc::new(RwLock::new(hecs::World::new()));
        let player = hecs::Entity::DANGLING;
        let textures = TextureManager::init("assets").await?;
        let keymap = load_keymap();
        let position_offsets = HashMap::new();
        let map = {
            let origin_pos = Vec2::ZERO;
            let height = 0;
            let width = 0;
            let tiles = Vec::new();
            Map { origin_pos, height, width, tiles }
        };

        Ok(Self {
            client,
			transport,
			world,
			player,
			textures,
			keymap,
            position_offsets,
            map,
        })
    }

    async fn run(&mut self) {
        prevent_quit();
        {
            let world = self.world.clone();
            std::thread::spawn(move || {
                Self::update_movement_loop(world);
            });
        }

        loop {
            let delta = Duration::from_secs_f32(get_frame_time());

            self.update(delta).await;

            if is_quit_requested() {
                break
            }

            self.render().await;

            next_frame().await;
        }
    }
    
    async fn update(&mut self, delta: Duration) {
        let mut messages = self.handle_input().await;

        if is_quit_requested() {
            self.handle_exit(&mut messages);
        }

        self.handle_network(delta, messages).await.expect("network error");
        self.handle_position_offsets(delta).await;
    }

    fn update_movement_loop(world: Arc<RwLock<hecs::World>>) {
        let delta_fix = Duration::from_millis(1000 / TICK_RATE);
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            let delta = now.duration_since(last_tick);
            last_tick = now;
            systems::update_movement(Arc::clone(&world), delta);
            std::thread::sleep(delta_fix - Instant::now().duration_since(last_tick));
        }
    }

    async fn handle_position_offsets(&mut self, delta: Duration) {
        self.position_offsets.retain(|id, offset| {
            let error_margin = offset.length();
            error_margin > 0.1 || (*id == self.player && error_margin > 0.4)
        });

        for (id, offset) in &mut self.position_offsets {
            let world = self.world.read().unwrap();
            if let Ok(mut transform) = world.get::<&mut Transform>(*id) {
                let change = offset.normalize() * 100.0 * delta.as_secs_f32();
                transform.pos += change;
                *offset -= change;
            }
        }
    }

    #[must_use]
    async fn handle_input(&mut self) -> Vec<ClientMessage> {
        let mut messages = Vec::new();

        let events: Vec<_> = get_keys_down()
            .iter()
            .filter_map(|key| self.keymap.get(key))
            .collect();

        let dir = {
            let up    = events.contains(&&Event::GoUp);
            let right = events.contains(&&Event::GoRight);
            let down  = events.contains(&&Event::GoDown);
            let left  = events.contains(&&Event::GoLeft);

            if      up && left    { Direction::NorthWest }
            else if up && right   { Direction::NorthEast }
            else if down && left  { Direction::SouthWest }
            else if down && right { Direction::SouthEast }
            else if up            { Direction::North }
            else if right         { Direction::East }
            else if down          { Direction::South }
            else if left          { Direction::West }
            else { Direction::None }
        };

        for event in events {
            match event {
                Event::Attack => {
                    println!("Attacking...")
                }
                // already handled
                Event::GoUp | Event::GoLeft | Event::GoDown | Event::GoRight => {}
            }
        }

        let movement = MovementState { dir };
        let now = utils::epoch_time_millis();

        messages.push(ClientMessage::Movement {
            movement: movement.clone(),
            timestamp: now,
        });

        self.handle_movement_update(
            self.player,
            movement,
            now,
        ).await;

        messages
    }

    async fn handle_movement_update
        (&mut self, entity: hecs::Entity, movement: MovementState, timestamp: u128)
    {
        let world = self.world.clone();
        std::thread::spawn(move || {
            sleep_until!(thread, timestamp + LATENCY);
            let world = world.read().unwrap();
            if let Ok(mut entity_movement) = world.get::<&mut MovementState>(entity) {
                *entity_movement = movement;
            }
        });
    }

    async fn handle_msg(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::InitialData { entity } => {
                self.player = entity;
            }

            ServerMessage::ClientDisconnect { entity } => {
                println!("disconnected: {entity:?}");
                let mut world = self.world.write().unwrap();
                let _ = world.despawn(entity);
            }

            ServerMessage::World(columns) => {
                for id in columns.ids {
                    let mut world = self.world.write().unwrap();
                    if !world.contains(id) {
                        world.spawn_at(id, ());
                    }
                }

                update_components!(self.world.write().unwrap(), columns, [
                    Player,
                    MovementState
                ]);

                for (id, real_transform) in columns.Transform {
                    let mut world = self.world.write().unwrap();
                    if let Ok(transform) = world.get::<&Transform>(id) {
                        let pos = transform.pos;
                        let real_pos = real_transform.pos;
                        self.position_offsets.insert(id, real_pos - pos);
                    } else {
                        world.insert(id, (real_transform,)).unwrap();
                    }
                }
            }
            ServerMessage::MapData { origin_pos, world_width, world_height } => {
                self.map.origin_pos = origin_pos;
                self.map.width = world_width;
                self.map.height = world_height;
                self.map.tiles = vec![Tile::empty(); world_width * world_height];
            }

            ServerMessage::MapPart { index, tiles } => {
                let end = index + tiles.len();
                if end > self.map.tiles.len() {
                    eprintln!(
                        "Map part {index}..{end} is outside the map ({} tiles)",
                        self.map.tiles.len()
                    );
                    return;
                }
                self.map.tiles[index..end].copy_from_slice(&tiles);
            }


            ServerMessage::Movement { entity, movement, timestamp } => {
                self.handle_movement_update(entity, movement, timestamp).await;
            }
        }
    }

    async fn handle_network
        (&mut self, delta: Duration, messages: Vec<ClientMessage>) -> Result<()>
    {
        self.client.update(delta);
        self.transport.update(delta, &mut self.client)?;

        if self.client.is_connected() {
            if !messages.is_empty() {
                self.client.send_message(
                    DefaultChannel::ReliableOrdered,
                    serialize(messages)?,
                );
            }

            for channel in RENET_CHANNELS {
                while let Some(ref data) = self.client.receive_message(*channel) {
                    match deserialize::<Vec<ServerMessage>>(data) {
                        Ok(messages) => {
                            for msg in messages {
                                self.handle_msg(msg).await;
                            }
                        }
                        Err(err) => eprintln!("{}", err)
                    }
                }
            }
        }

        self.transport.send_packets(&mut self.client)?;

        Ok(())
    }

    fn handle_exit(&self, messages: &mut Vec<ClientMessage>) {
        messages.push(ClientMessage::Disconnect);
    }

    async fn render_map(&self, viewport_offset: Vec2) {
        const UNKNOWN_TEXTURE: &'static [u8] = &[
            //  R,   G, B,   A,
            0,   0, 0,   255, // BLACK
            255, 0, 255, 255, // PURPLE
            255, 0, 255, 255, // PURPLE
            0,   0, 0,   255, // BLACK
        ];

        if self.map.width == 0 {
            return;
        }

        let unknown = Texture2D::from_rgba8(2, 2, UNKNOWN_TEXTURE);
        let tile_size = vec2(TILE_SIZE, TILE_SIZE);

        for (idx, tile) in self.map.tiles.iter().enumerate() {
            let x = idx % self.map.width;
            let y = idx / self.map.width;
            let pos = self.map.origin_pos
            + vec2(x as f32, y as f32) * TILE_SIZE
            - viewport_offset;

            'render_back: {
                let back_path = match tile.back {
                    BackTile::Grass => "back/grass.png",
                    BackTile::Stone => "back/stone.png",
                    BackTile::None => { break 'render_back }
                };

                let back = self.textures.get(back_path).await.unwrap_or(&unknown);
                draw_texture_ex(
                    back,
                    pos.x,
                    pos.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(tile_size),
                        ..Default::default()
                    },
                );
            }

            'render_front: {
                let path = match tile.front {
                    FrontTile::Tree  => "front/tree.png",
                    FrontTile::Wall  => "front/wall.png",
                    FrontTile::Stone => "front/stone.png",
                    FrontTile::None => { break 'render_front }
                };

                let front = self.textures.get(path).await.unwrap_or(&unknown);
                draw_texture_ex(
                    front,
                    pos.x,
                    pos.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(tile_size),
                        ..Default::default()
                    },
                );
            }
        }
    }


    async fn render(&self) {
        clear_background(LIGHTGRAY);

        if !self.client.is_connected() {
            let text = "Connecting...";
            const FONT_WIDTH: f32 = 17.5;
            const FONT_HEIGHT: f32 = 20.0;
            draw_text(
                text,
                (screen_width() - text.len() as f32 * FONT_WIDTH) / 2.,
                (screen_height() + FONT_HEIGHT) / 2.,
                40.0,
                BLACK,
            );
        }

        let viewport_offset = {
            let world = self.world.read().unwrap();
            match world.get::<&Transform>(self.player) {
                Ok(player_transform) => {
                    player_transform.pos
                    - vec2(screen_width(), screen_height())
                    / 2.0
                }
                Err(_) => Vec2::ZERO,
            }
        };

        self.render_map(viewport_offset).await;

        {
            let world = self.world.read().unwrap();
            for (entity, transform, movement) in
                world.query::<(hecs::Entity, &Transform, &MovementState)>().iter()
            {
                let color = if entity == self.player { GREEN } else { RED };

                let player_x = transform.pos.x - viewport_offset.x;
                let player_y = transform.pos.y - viewport_offset.y;

                let dir = Vec2::from(&movement.dir).normalize() * ENTITY_SPRITE_SIZE;

                const ARROW_SIZE: f32 = 0.4;
                const ARROW_COLOR: Color = GRAY;
                let arrow_point = vec2(
                    player_x + ENTITY_SPRITE_SIZE / 2. + dir.x, 
                    player_y + ENTITY_SPRITE_SIZE / 2. + dir.y,
                );

                draw_line(
                    player_x + ENTITY_SPRITE_SIZE / 2.,
                    player_y + ENTITY_SPRITE_SIZE / 2.,
                    arrow_point.x,
                    arrow_point.y,
                    3.5,
                    ARROW_COLOR,
                );

                draw_triangle(
                    arrow_point + dir / 2.,
                    arrow_point - dir.perp() * ARROW_SIZE / 2.,
                    arrow_point + dir.perp() * ARROW_SIZE / 2.,
                    ARROW_COLOR,
                );

                draw_rectangle(
                    player_x,
                    player_y,
                    ENTITY_SPRITE_SIZE,
                    ENTITY_SPRITE_SIZE,
                    color,
                );
            }
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: "Slottet".to_string(),
        window_width: 800,
        window_height: 600,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() -> Result<()> {
    let args: &[String] = &env::args().collect::<Vec<_>>().into_boxed_slice();
    let [_, addr] = args else {
        eprintln!("help: {bin} [ip:port]", bin=args[0]);
        return Ok(());
    };

    let mut client = Client::connect(addr.clone()).await?;
    client.run().await;

    Ok(())
}
