#![allow(dead_code, unused_imports)]
mod exchange;
use iced::widget::{
    self, column, container, horizontal_space, image, row, text, vertical_space, Column,
};
use iced::{window, Application, Color, Command, Element, Length, Settings, Theme};
use reqwest::Client;
use toml::value::Table;

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
        default_font: Some(include_bytes!("./YangRenDongZhuShiTi-Light-2.ttf")),
        ..Settings::default()
    })
}

#[derive(Debug)]
enum Memories {
    Loading,       // 有加载任务尚未完成
    Loaded(State), // 已完成加载，等待下个事件
}

#[derive(Clone, Debug)]
pub struct State {
    stage: Stage,
    idxtable: Table,
    client: Client,
    url_prefix: String,
    storage: String,
}

#[derive(Clone, Debug)]
enum Stage {
    ChoosingCharacter(ChoosingState),
    ShowingPlots,
    Graduated,
}

#[derive(Clone, Debug, Default)]
pub struct ChoosingState {
    on_image: u32,
    image: Option<image::Handle>,
    // 当前图片为 idx[on_image]
}

#[derive(Debug)]
enum Message {
    Loaded(Result<State, Error>),
    FetchImage(Result<State, Error>),
    LoadedImage(Result<ChoosingState, Error>),
    ImageChanged(u32),
}

#[derive(Debug)]
pub enum Error {
    APIError,
    LanguageError,
    ParseError,
}

impl Application for Memories {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Memories, Command<Message>) {
        (
            Memories::Loading,
            Command::perform(State::get_idx(), Message::FetchImage),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Memories::Loading => "加载中",
            Memories::Loaded(_) => "加载完毕！",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        println!("On update().");
        match self {
            Memories::Loading => match message {
                Message::FetchImage(Ok(state)) => {
                    *self = Memories::Loading;
                    Command::perform(exchange::load_image(state), Message::Loaded)
                }
                Message::Loaded(Ok(state)) => {
                    *self = Memories::Loaded(state);
                    Command::none()
                }
                _ => {
                    println!("Error Processing: {:#?}", message);
                    Command::none()
                }
            },
            Memories::Loaded(state) => {
                println!("Loaded...");
                match &state.stage {
                    Stage::ChoosingCharacter(chosen) => match message {
                        Message::ImageChanged(next) => {
                            println!("On Loaded-ChoosingCharacter-ImageChanged");
                            Command::none()
                        }
                        _ => {
                            println!("Not `ImageChanged` message");
                            Command::none()
                        }
                    },
                    _ => {
                        println!("Some other message.");
                        Command::none()
                    }
                }
            }
        }
    }
    fn view(&self) -> Element<Message> {
        println!("On view()...");
        match self {
            Memories::Loading => {
                println!("On Memories::Loading");
                container(
                    column![
                        text("正在加载中  Loading...").size(40),
                        text("有你，才是一班。").size(20)
                    ]
                    .width(Length::Shrink),
                )
            }
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into(),
            Memories::Loaded(state) => {
                println!("Loaded Image!");
                match &state.stage {
                    Stage::ChoosingCharacter(chosen) => row![
                        match &chosen.image {
                            Some(handle) => Element::from(image::viewer(handle.clone())),
                            None => Element::from(text("Not able to load image.").size(40)),
                        },
                        Self::show_name("Class 1")
                    ]
                    .into(),
                    _ => row![Self::show_name("Not implemented")].into(),
                }
            }
        }
        /*let content = row![
            image("data/image/grade7/开学合照.jpg"),
            /*horizontal_space(Length::Units(20)),
            Self::show_profile("abc")*/
        ];
        container(content).into()*/
    }
}
impl Memories {
    fn show_name(title: &str) -> Column<Message> {
        column![text(title).size(50)].spacing(20)
    }
}
