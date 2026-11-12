use crate::components;
use crate::components::MoveDirection;
use crate::components::MovementUpdate;

use serde::{ Serialize, Deserialize };

// from server
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    World(Columns),
    InitialData {
        entity: hecs::Entity,
    },
    MoveDirection {
        entity: hecs::Entity,
        dir: MoveDirection,
        timestamp: f64,
    },
}

// from client
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    MoveDirection {
        move: MovementUpdate,
    },
}

type Column<T> = Vec<(hecs::Entity, T)>;

#[allow(non_snake_case)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    pub ids: Vec<hecs::Entity>,
    pub Player: Column<components::Player>,
    pub Transform: Column<components::Transform>,
    pub MoveDirection: Column<components::MoveDirection>,
}

pub const NUM_COLUMNS: u64 = 3;

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
