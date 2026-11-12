mod texture;
mod keybinds;

use crate::texture::TextureManager;
use crate::keybinds::{ Event, KeyMap, load_keymap };
use common::{ Result, LATENCY };
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
use std::time::Duration;
use renet::{ RenetClient, DefaultChannel };
use renet_netcode::NetcodeClientTransport;

struct Client {
    client: RenetClient,
    transport: NetcodeClientTransport,
    world: hecs::World,
    player: hecs::Entity,
    keymap: KeyMap,
    textures: TextureManager,
    message_buffer: Vec<ServerMessage>,
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
    pub async fn connect(addr: String) -> Result<Self> {
        let (client, transport) = create_client(addr)?;
        let world = hecs::World::new();
        let player = hecs::Entity::DANGLING;

        let textures = TextureManager::init("assets").await?;
        let keymap = load_keymap();
        let message_buffer = Vec::new();

        Ok(Self {
            client,
			transport,
			world,
			player,
			textures,
			keymap,
            message_buffer,
        })
    }

    pub async fn update(&mut self, delta: Duration) {
        let messages = self.handle_input().await;
        self.handle_network(delta, messages).await;
        let now = utils::epoch_time();
        for (i, msg) in self.message_buffer.iter().enumerate() {
            match msg {
                ServerMessage::Movement { entity, movement, timestamp } => {
                    let dt = now - timestamp;
                    if dt >= LATENCY {
                        println!("{dt}");
                        if let Ok(mut move_state) = self.world.get::<&mut MovementState>(*entity) {
                            *move_state = movement.clone();
                        }
                        self.message_buffer.swap_remove(i);
                        break
                    }
                }
                _ => {}
            }
        }
        systems::update_movement(&mut self.world, delta).await;
    }

    async fn handle_input(&mut self) -> Vec<ClientMessage> {
        let events: Vec<_> = get_keys_down()
            .iter()
            .filter_map(|key| self.keymap.get(key))
            .collect();

        let mut messages = Vec::new();

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
                Event::Attack  => println!("Attacking enemy..."),

                // already handled
                Event::GoUp | Event::GoLeft | Event::GoDown | Event::GoRight => {}
            }
        }

        if let Ok(mut move_state) = self.world.get::<&mut MovementState>(self.player) {
            if !(move_state.dir == dir) {
                move_state.dir = dir.clone();
                messages.push(ClientMessage::Movement {
                    movement: MovementState { dir },
                    timestamp: utils::epoch_time(),
                });
            }
        }

        messages
    }

    async fn handle_msg(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::InitialData { entity } => {
                self.player = entity;
            }

            ServerMessage::World(columns) => {
                for id in columns.ids {
                    if !self.world.contains(id) {
                        self.world.spawn_at(id, ());
                    }
                }

                update_components!(self.world, columns, [
                    Player,
                    MovementState
                ]);

                for (id, transform) in columns.Transform {
                    let do_move = id != self.player || match self.world.get::<&Transform>(self.player) {
                        Err(_) => true,
                        Ok(_entity_transform) => {
                            // TODO check diff
                            // println!("diff: {}", transform.pos.distance(entity_transform.pos));
                            false
                        }
                    };

                    if do_move {
                        self.world.insert(id, (transform,)).unwrap();
                    }
                }
            }

            ServerMessage::Movement { .. } => {
                self.message_buffer.push(msg);
            }
        }
    }

    async fn handle_network(&mut self, delta: Duration, messages: Vec<ClientMessage>) {
        self.client.update(delta);
        self.transport.update(delta, &mut self.client).unwrap();

        if self.client.is_connected() {
            if !messages.is_empty() {
                self.client.send_message(
                    DefaultChannel::ReliableOrdered,
                    serialize(messages).unwrap()
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

        self.transport.send_packets(&mut self.client).unwrap();
    }

    pub async fn render(&self) {
        clear_background(LIGHTGRAY);

        if !self.client.is_connected() {
            let text = "Connecting...";
            const FONT_WIDTH: f32 = 17.5;
            const FONT_HEIGHT: f32 = 20.0;
            draw_text(text, (screen_width() - text.len() as f32 * FONT_WIDTH) / 2., (screen_height() + FONT_HEIGHT) / 2., 40.0, BLACK);
        }

        let viewport_offset = match self.world.get::<&Transform>(self.player) {
            Ok(player_transform) => {
                player_transform.pos
                - vec2(screen_width(), screen_height())
                / 2.0
            }
            Err(_) => Vec2::ZERO,
        };

        for (entity, transform) in
            self.world.query::<(hecs::Entity, &Transform)>().iter()
        {
            let color = if entity == self.player { GREEN } else { RED };

            draw_rectangle(
                transform.pos.x - viewport_offset.x,
                transform.pos.y - viewport_offset.y,
                30.0,
                30.0,
                color,
            );
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

    loop {
        let delta = get_frame_time();

        client.update(Duration::from_secs_f32(delta)).await;

        client.render().await;

        next_frame().await
    }
}
