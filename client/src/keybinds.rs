use macroquad::input::{ KeyCode };

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Jump,
}

lazy_static! {
    static ref keymap: HashMap<KeyCode, Event> = {
        HashMap::from([
            (KeyCode::Space, Event::Jump),
        ])
    };
}
