use macroquad::input::KeyCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Attack,

    GoUp,
    GoLeft,
    GoDown,
    GoRight,
}

pub type KeyMap = HashMap<KeyCode, Event>;

#[inline]
pub fn load_keymap() -> KeyMap {
    HashMap::from([
        (KeyCode::Space, Event::Attack),

        (KeyCode::W, Event::GoUp),
        (KeyCode::A, Event::GoLeft),
        (KeyCode::S, Event::GoDown),
        (KeyCode::D, Event::GoRight),
    ])
}
