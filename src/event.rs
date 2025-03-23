use std::collections::HashMap;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    Quit,
    Down,
    Up,
    ToggleAction,
    Execute,
}

#[derive(Debug, Default)]
pub struct UserEventMapper {
    map: HashMap<KeyEvent, UserEvent>,
}

impl UserEventMapper {
    #[rustfmt::skip]
    pub fn new() -> UserEventMapper {
        let mut map = HashMap::new();
        map.insert(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), UserEvent::Quit);
        map.insert(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), UserEvent::Quit);
        map.insert(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), UserEvent::Down);
        map.insert(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL), UserEvent::Down);
        map.insert(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), UserEvent::Up);
        map.insert(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL), UserEvent::Up);
        map.insert(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), UserEvent::ToggleAction);
        map.insert(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), UserEvent::Execute);
        UserEventMapper { map }
    }

    pub fn find_event(&self, key: KeyEvent) -> Option<UserEvent> {
        self.map.get(&key).copied()
    }
}
