use crate::Message;
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
            KeyCode::Plus | KeyCode::NumpadAdd => Some(Message::ScaleEnlarge),
            KeyCode::Minus | KeyCode::NumpadSubtract => Some(Message::ScaleDown),
            KeyCode::Equals | KeyCode::NumpadEquals => Some(Message::ScaleRestore),
            KeyCode::O => Some(Message::OpenUrl(None)),
            KeyCode::PlayPause | KeyCode::M => Some(Message::SwitchMusicStatus),
            KeyCode::N => Some(Message::NextSong),
            KeyCode::R => Some(Message::Refresh),
            KeyCode::E => Some(Message::OpenSettings),
            KeyCode::Escape => Some(Message::EscapeFullScreen),
            KeyCode::Enter => {
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
                with_key!(KeyCode::Enter) => Some(Message::NextStage),
                with_key!(KeyCode::Left) | with_key!(KeyCode::A) => Some(Message::PreviousEvent),
                with_key!(KeyCode::Right) | with_key!(KeyCode::D) => Some(Message::NextEvent),
                with_key!(KeyCode::Up) | with_key!(KeyCode::W) => Some(Message::PreviousPhoto),
                with_key!(KeyCode::Down) | with_key!(KeyCode::S) => Some(Message::NextPhoto),
                with_key!(KeyCode::Space) => Some(Message::NextEvent),
                keyboard::Event::KeyPressed {
                    key_code: KeyCode::Tab,
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
                with_key!(KeyCode::Space) => Some(Message::NextStage),
                keyboard::Event::KeyPressed {
                    key_code: KeyCode::Tab,
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
                with_key!(KeyCode::Enter) => Some(Message::NextStage),
                with_key!(KeyCode::Left) | with_key!(KeyCode::A) => Some(Message::PreviousEvent),
                with_key!(KeyCode::Right) | with_key!(KeyCode::D) => Some(Message::NextEvent),
                with_key!(KeyCode::Up) | with_key!(KeyCode::W) => Some(Message::PreviousPhoto),
                with_key!(KeyCode::Down) | with_key!(KeyCode::S) => Some(Message::NextPhoto),
                with_key!(KeyCode::Space) => Some(Message::NextEvent),
                keyboard::Event::KeyPressed {
                    key_code: KeyCode::Tab,
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

#[cfg(target_os = "windows")]
pub fn open_url(mut filename: String) {
    filename = format!("\x22{filename}\x22"); // \x22 为英文双引号
    std::process::Command::new("powershell")
        .args(["start", filename.as_str()])
        .output()
        .unwrap();
}

#[cfg(target_os = "macos")]
pub fn open_url(mut filename: String) {
    filename = format!("\x22{filename}\x22");
    std::process::Command::new("open")
        .arg(filename)
        .output()
        .unwrap();
}

#[cfg(not(target_os = "windows", target_os = "macos"))]
pub fn open_url(mut filename: String) {
    filename = format!("\x22{filename}\x22");
    std::process::Command::new("xdg-open")
        .arg(filename)
        .output()
        .unwrap();
}
