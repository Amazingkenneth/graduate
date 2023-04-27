use crate::Message;
use crate::State;
use iced::keyboard::{self, KeyCode};
use iced::Event;

macro_rules! with_key {
    ($key: path) => {
        keyboard::Event::KeyPressed {
            key_code: $key,
            modifiers: _,
        }
    };
}

fn global_response(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed {
            modifiers: m,
            key_code,
        } => match key_code {
            keyboard::KeyCode::Plus | keyboard::KeyCode::NumpadAdd => Some(Message::ScaleEnlarge),
            keyboard::KeyCode::Minus | keyboard::KeyCode::NumpadSubtract => {
                Some(Message::ScaleDown)
            }
            keyboard::KeyCode::Equals | keyboard::KeyCode::NumpadEquals => {
                Some(Message::ScaleRestore)
            }
            keyboard::KeyCode::O => Some(Message::OpenInExplorer),
            keyboard::KeyCode::PlayPause => Some(Message::SwitchMusicStatus),
            keyboard::KeyCode::R => Some(Message::Refresh),
            keyboard::KeyCode::S => Some(Message::OpenSettings),
            keyboard::KeyCode::Escape => Some(Message::EscapeFullScreen),
            keyboard::KeyCode::Enter => {
                if m.alt() {
                    Some(Message::ToggleMode)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
}

pub fn on_loading(event: Event, _: iced::event::Status) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard_event) => {
            if let Some(ret) = global_response(keyboard_event) {
                Some(ret)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn on_entry_state(event: Event, _: iced::event::Status) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard_event) => {
            if let Some(ret) = global_response(keyboard_event) {
                return Some(ret);
            }
            match keyboard_event {
                with_key!(keyboard::KeyCode::Enter) => Some(Message::NextStage),
                with_key!(KeyCode::Left) => Some(Message::PreviousEvent),
                with_key!(KeyCode::Right) => Some(Message::NextEvent),
                with_key!(keyboard::KeyCode::Up) => Some(Message::PreviousPhoto),
                with_key!(keyboard::KeyCode::Down) => Some(Message::NextPhoto),
                with_key!(keyboard::KeyCode::Space) => Some(Message::NextEvent),
                keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                } => Some(if modifiers.shift() {
                    Message::PreviousEvent
                } else {
                    Message::NextEvent
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn on_choosing_character(event: Event, _: iced::event::Status) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard_event) => {
            if let Some(ret) = global_response(keyboard_event) {
                return Some(ret);
            }
            match keyboard_event {
                with_key!(KeyCode::Left) => Some(Message::PreviousPerson),
                with_key!(KeyCode::Right) => Some(Message::NextPerson),
                with_key!(keyboard::KeyCode::Space) => Some(Message::NextStage),
                keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                } => Some(if modifiers.shift() {
                    Message::PreviousPerson
                } else {
                    Message::NextPerson
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn on_graduation(event: Event, _: iced::event::Status) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard_event) => {
            if let Some(ret) = global_response(keyboard_event) {
                return Some(ret);
            }
            match keyboard_event {
                with_key!(keyboard::KeyCode::Enter) => Some(Message::NextStage),
                with_key!(KeyCode::Left) => Some(Message::PreviousEvent),
                with_key!(KeyCode::Right) => Some(Message::NextEvent),
                with_key!(keyboard::KeyCode::Up) => Some(Message::PreviousPhoto),
                with_key!(keyboard::KeyCode::Down) => Some(Message::NextPhoto),
                with_key!(keyboard::KeyCode::Space) => Some(Message::NextEvent),
                keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                } => Some(if modifiers.shift() {
                    Message::PreviousEvent
                } else {
                    Message::NextEvent
                }),
                _ => None,
            }
        }
        _ => None,
    }
}
