use protocol::{ Result };
use protocol::network::setup;
use protocol::components::{ Player, Transform };

use renet::{ RenetClient };
use renet_netcode::{ NetcodeClientTransport };

use macroquad::prelude::*;
use hecs;

pub fn connect(addr: String) -> Result<(RenetClient, NetcodeClientTransport, hecs::World, hecs::Entity)> {
    let (client, transport) = setup::create_client(addr)?;
    // TODO get world and id from server.
    
    let mut ecs = hecs::World::new();
    let player = ecs.spawn((
        Player {},
        Transform {
            position: vec2(1., 4.)
        }
    ));

    ecs.spawn(( Player {}, Transform { position: vec2(60., 250.) }));
    ecs.spawn(( Player {}, Transform { position: vec2(-110., 70.) }));
    ecs.spawn(( Player {}, Transform { position: vec2(30., 40.) }));


    Ok((
        client,
        transport,
        ecs,
        player,
    ))
}

