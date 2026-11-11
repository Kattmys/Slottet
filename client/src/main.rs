use std::{
    env,
    time::Duration,
};
use common::{
    Result,
    network::*,
    components::*,
};
use renet::RenetClient;
use renet_netcode::NetcodeClientTransport;
use macroquad::prelude::*;

struct Client {
    client: RenetClient,
    transport: NetcodeClientTransport,
    world: hecs::World,
    player: hecs::Entity,
}

impl Client {
    fn connect(addr: String) -> Result<Self> {
        let (client, transport) = create_client(addr)?;
        let world = hecs::World::new();
        let player = hecs::Entity::DANGLING;

        Ok(Self { client, transport, world, player })
    }

    async fn update(&mut self, delta: Duration) {
        self.handle_network(delta).await;
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

                for (id, obj) in columns.Player {
                    self.world.insert(id, (obj,)).unwrap();
                }

                for (id, obj) in columns.Transform {
                    self.world.insert(id, (obj,)).unwrap();
                }
            }
        }
    }

    async fn handle_network(&mut self, delta: Duration) {
        self.client.update(delta);
        self.transport.update(delta, &mut self.client).unwrap();

        if self.client.is_connected() {
            // self.client.send_message(DefaultChannel::ReliableOrdered,
            //     serialize(send_msgs).unwrap()
            // );

            for channel in RENET_CHANNELS {
                while let Some(ref data) = self.client.receive_message(*channel) {
                    match deserialize::<ServerMessages>(data) {
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

    async fn render(&self) {
        clear_background(LIGHTGRAY);

        if !self.client.is_connected() {
            draw_text("Connecting...", 10.0, 40.0, 40.0, BLACK);
        }

        let viewport_offset = match self.world.get::<&Transform>(self.player) {
            Ok(player_transform) =>
                player_transform.pos - vec2(
                    screen_width(),
                    screen_height()
                ) / 2.0,
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

    let mut client = Client::connect(addr.clone())?;

    loop {
        let delta = get_frame_time();

        client.update(Duration::from_secs_f32(delta)).await;

        client.render().await;

        next_frame().await
    }
}

