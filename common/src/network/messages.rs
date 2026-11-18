use crate::components;
use crate::components::MovementState;
use crate::map::Tile;

use macroquad::math::Vec2;
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    World(Columns),
    InitialData {
        entity: hecs::Entity,
    },
    Movement {
        entity: hecs::Entity,
        movement: MovementState,
        timestamp: u128,
    },
    ClientDisconnect {
        entity: hecs::Entity,
    },
    MapData {
        origin_pos: Vec2,
        world_width: usize,
        world_height: usize,
    },
    MapPart {
        index: usize,
        tiles: Box<[Tile]>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Movement {
        movement: MovementState,
        timestamp: u128,
    },
    Disconnect,
}

type Column<T> = Vec<(hecs::Entity, T)>;

#[allow(non_snake_case)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    pub ids: Vec<hecs::Entity>,
    pub Player: Column<components::Player>,
    pub Transform: Column<components::Transform>,
    pub MovementState: Column<components::MovementState>,
    pub Inventory: Column<components::Inventory>,
}

pub const NUM_COLUMNS: u64 = 4;

#[macro_export]
macro_rules! columns {
    ($world:expr, [$($ident:ident),*]) => {{
        let mut columns = common::network::Columns::default();
        $(
            columns.$ident =
                $world.query::<(hecs::Entity, &$ident)>().iter()
                // TODO do not clone
                .map(|(id, obj)| (id, obj.clone()))
                .collect::<Vec<_>>();
        )*
        $(
            for (id, _) in &columns.$ident {
                if !columns.ids.contains(&id) {
                    columns.ids.push(id.clone());
                }
            }
        )*
        columns
    }}
}
