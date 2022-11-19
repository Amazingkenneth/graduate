#![allow(dead_code, unused_imports)]
mod exchange;
// use iced::futures;
use iced::widget::{column, container, horizontal_space, image, row, text, vertical_space, Column};
use iced::{window, Application, Color, Command, Element, Length, Settings, Theme};
use toml::value::Datetime;

const INITIAL_WIDTH: u32 = 1400;
const INITIAL_HEIGHT: u32 = 800;
pub enum LayoutDirection {
    Horizontal,
    Upright,
}
pub fn get_dir(width: u32, height: u32) -> LayoutDirection {
    let upr = width * INITIAL_HEIGHT > height * INITIAL_WIDTH;
    match upr {
        true => LayoutDirection::Upright,
        false => LayoutDirection::Horizontal,
    }
}

pub fn main() -> iced::Result {
    Memories::run(Settings {
        window: window::Settings {
            size: (INITIAL_WIDTH, INITIAL_HEIGHT),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

enum Memories {
    Loading,
    // Errored,
    ChoosingCharacter(ChoosingState),
    ShowingPlots,
    Graduated,
}

pub struct ChoosingState {
    chosen_character: u32,
    on_image: u32,
    idx: Vec<EntryImage>,
    // 当前图片为 idx[on_image]
}
pub struct EntryImage {
    name: String,
    description: String,
    date: Datetime,
    image: image::Handle,
}

#[derive(Debug)]
enum Message {
    ImageChanged(u32),
}

impl Application for Memories {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Memories, Command<Message>) {
        (Memories::Loading, Command::none())
    }
    fn title(&self) -> String {
        let subtitle = match self {
            Memories::Loading => "加载中",
            _ => "Whoops!",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Memories::ChoosingCharacter(s) => match message {
                Message::ImageChanged(to) => {
                    s.on_image = to;
                }
            },
            _ => {}
        }
        Command::none()
    }
    fn view(&self) -> Element<Message> {
        let content = row![
            image("data/image/grade7/开学合照.jpg"),
            /*horizontal_space(Length::Units(20)),
            Self::show_profile("abc")*/
        ];
        container(content).into()
    }
}
impl Memories {
    fn show_profile(title: &str) -> Column<Message> {
        column![text(title).size(50)].spacing(20)
    }
}
