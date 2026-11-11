use crate::components;
use macroquad::math::{ Vec2 };
use serde::{ Serialize, Deserialize };

pub type ServerMessages = Vec<ServerMessage>;
pub type ClientMessages = Vec<ClientMessage>;

// from server
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    World(Columns),
    InitialData {
        entity: hecs::Entity,
    },
}

// from client
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Velocity(Vec2),
}

type Column<T> = Vec<(hecs::Entity, T)>;

#[allow(non_snake_case)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Columns {
    pub ids: Vec<hecs::Entity>,
    pub Player: Column<components::Player>,
    pub Transform: Column<components::Transform>,
}

#[macro_export]
macro_rules! columns {
    ($world:expr, [$($ident:ident),*]) => {{
        let mut columns = protocol::network::Columns::default();
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
