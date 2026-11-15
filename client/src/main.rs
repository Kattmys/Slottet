mod texture;
mod keybinds;

use crate::texture::TextureManager;
use crate::keybinds::{ Event, KeyMap, load_keymap };
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
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use renet::{ RenetClient, DefaultChannel };
use renet_netcode::NetcodeClientTransport;

struct Client {
    client: RenetClient,
    transport: NetcodeClientTransport,
    world: Arc<Mutex<hecs::World>>,
    player: hecs::Entity,
    keymap: KeyMap,
    textures: TextureManager,
    position_offsets: HashMap<hecs::Entity, Vec2>,
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
        let world = Arc::new(Mutex::new(hecs::World::new()));
        let player = hecs::Entity::DANGLING;
        let textures = TextureManager::init("assets").await?;
        let keymap = load_keymap();
        let position_offsets = HashMap::new();

        Ok(Self {
            client,
			transport,
			world,
			player,
			textures,
			keymap,
            position_offsets,
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

            let mut messages = self.handle_input().await;

            if is_quit_requested() {
                self.handle_exit(&mut messages);
            }

            self.handle_network(delta, messages).await.expect("network error");

            if is_quit_requested() {
                break
            }

            self.handle_position_offsets(delta).await;

            self.render().await;

            next_frame().await;
        }
    }

    fn update_movement_loop(world: Arc<Mutex<hecs::World>>) {
        let delta_fix = Duration::from_millis(1000 / TICK_RATE);
        let mut last_tick = Instant::now();

        loop {
            let now = Instant::now();
            let delta = now.duration_since(last_tick);
            last_tick = now;
            {
                let mut world = world.lock().unwrap();
                systems::update_movement(&mut world, delta);
            }
            std::thread::sleep(delta_fix - Instant::now().duration_since(last_tick));
        }
    }

    async fn handle_position_offsets(&mut self, delta: Duration) {
        self.position_offsets.retain(|_id, offset| offset.length() > 0.1);
        for (id, offset) in &mut self.position_offsets {
            let world = self.world.lock().unwrap();
            if let Ok(mut transform) = world.get::<&mut Transform>(*id) {
                let change = offset.normalize() * 100.0 * delta.as_secs_f32();
                transform.pos += change;
                *offset -= change;
            }
        }
    }

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
                Event::Attack  => println!("Attacking enemy..."),

                // already handled
                Event::GoUp | Event::GoLeft | Event::GoDown | Event::GoRight => {}
            }
        }

        let movement = MovementState { dir };

        messages.push(ClientMessage::Movement {
            movement: movement.clone(),
            timestamp: utils::epoch_time_millis(),
        });

        self.handle_movement_update(
            self.player,
            movement,
            utils::epoch_time_millis()
        ).await;

        messages
    }

    async fn handle_movement_update
        (&mut self, entity: hecs::Entity, movement: MovementState, timestamp: u128)
    {
        let world = self.world.clone();
        std::thread::spawn(move || {
            sleep_until!(thread, timestamp + LATENCY);
            let world = world.lock().unwrap();
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
                let mut world = self.world.lock().unwrap();
                let _ = world.despawn(entity);
            }

            ServerMessage::World(columns) => {
                for id in columns.ids {
                    let mut world = self.world.lock().unwrap();
                    if !world.contains(id) {
                        world.spawn_at(id, ());
                    }
                }

                update_components!(self.world.lock().unwrap(), columns, [
                    Player,
                    MovementState
                ]);

                for (id, real_transform) in columns.Transform {
                    let mut world = self.world.lock().unwrap();
                    if let Ok(transform) = world.get::<&Transform>(id) {
                        let pos = transform.pos;
                        let real_pos = real_transform.pos;
                        self.position_offsets.insert(id, real_pos - pos);
                    } else {
                        world.insert(id, (real_transform,)).unwrap();
                    }
                }
            }

            ServerMessage::Movement { entity, movement, timestamp } => {
                self.handle_movement_update(entity, movement, timestamp).await;
            }
        }
    }

    async fn handle_network(&mut self, delta: Duration, messages: Vec<ClientMessage>) -> Result<()> {
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

    async fn render(&self) {
        clear_background(LIGHTGRAY);

        if !self.client.is_connected() {
            let text = "Connecting...";
            const FONT_WIDTH: f32 = 17.5;
            const FONT_HEIGHT: f32 = 20.0;
            draw_text(text, (screen_width() - text.len() as f32 * FONT_WIDTH) / 2., (screen_height() + FONT_HEIGHT) / 2., 40.0, BLACK);
        }

        let viewport_offset = {
            let world = self.world.lock().unwrap();
            match world.get::<&Transform>(self.player) {
                Ok(player_transform) => {
                    player_transform.pos
                    - vec2(screen_width(), screen_height())
                    / 2.0
                }
                Err(_) => Vec2::ZERO,
            }
        };

        {
            let world = self.world.lock().unwrap();
            for (entity, transform) in
                world.query::<(hecs::Entity, &Transform)>().iter()
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
