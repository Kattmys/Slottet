pub mod network;
pub mod texture;
use crate::network::{ connect };

use protocol::components::{ Transform };

use std::env;

use macroquad::prelude::*;
use hecs;

fn conf() -> Conf {
    Conf {
        window_title: "Slottet".to_string(),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let args: &[String] = &env::args().collect::<Vec<_>>().into_boxed_slice();
    let [_, addr] = args else {
        eprintln!("help: {bin} [ip:port]", bin=args[0]);
        return;
    };

    let (_server, _transport, ecs, player) = connect(addr.to_string()).unwrap();

    let textures = texture::TextureManager::init("assets").await.unwrap();

    loop {
        clear_background(LIGHTGRAY);

        draw_text("HELLO", 20.0, 20.0, 30.0, DARKGRAY);

        render(&ecs, player, &textures).await;

        next_frame().await
    }
}

pub async fn render(ecs: &hecs::World, player: hecs::Entity, textures: &texture::TextureManager) {
    let viewport_offset = match ecs.get::<&Transform>(player) {
        Ok(player_transform)
            => player_transform.position - vec2(screen_width() / 2., screen_height() / 2.),
        Err(_) => Vec2::ZERO,
    };

    let enemy_texture = textures.get("pfp.png").await.unwrap();
    for (entity, transform) in ecs.query::<(hecs::Entity, &Transform)>().iter() {
        let _color = if entity == player {
            draw_rectangle(
                transform.position.x - viewport_offset.x,
                transform.position.y - viewport_offset.y,
                30.0,
                30.0,
                GREEN,
            );
            GREEN
        } else {
            draw_texture(
                enemy_texture,
                transform.position.x - viewport_offset.x,
                transform.position.y - viewport_offset.y,
                WHITE
            );
            RED
        };

    }
}
