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

/*pub enum LayoutDirection {
    Horizontal,
    Upright,
}
pub fn get_dir(width: u32, height: u32) -> LayoutDirection {
    let upr = width * INITIAL_HEIGHT > height * INITIAL_WIDTH;
    match upr {
        true => LayoutDirection::Upright,
        false => LayoutDirection::Horizontal,
    }
}*/

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
    on_event: u32,
    on_image: u32,
    image: Option<image::Handle>,
    // 当前图片为 idx[on_image]
}

#[derive(Clone, Debug)]
enum Message {
    Loaded(Result<State, Error>),
    FetchImage(Result<State, Error>),
    LoadedImage(Result<ChoosingState, Error>),
    NextEvent,
}

#[derive(Clone, Debug)]
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
        println!("On update()");
        match self {
            Memories::Loading => match message {
                Message::FetchImage(Ok(state)) => {
                    // *self = Memories::Loading;
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
                match state.stage {
                    Stage::ChoosingCharacter(ref chosen) => match message {
                        Message::NextEvent => {
                            let to: i64 = (chosen.on_event + 1).into();
                            let state = state.clone();
                            *self = Memories::Loading;
                            Command::perform(exchange::change_image(state, to), Message::Loaded)
                        }
                        Message::Loaded(_) => {
                            println!("On Message::Loaded");
                            Command::none()
                        }
                        _ => {
                            println!("Not regular message");
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
        println!("On view()... self = {:?}", self);
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
                            Some(handle) => {
                                println!("handle: {:?}", handle);
                                Element::from(image::viewer(handle.clone()))
                            }
                            None => Element::from(text("Not able to load image.").size(40)),
                        },
                        column![
                            text(
                                state
                                    .get_current_event(chosen.on_event)
                                    .get("description")
                                    .expect("No image value in the item.")
                                    .as_str()
                                    .expect("cannot convert into &str")
                            )
                            .size(50),
                            widget::Button::new(widget::Svg::new(
                                widget::svg::Handle::from_memory(
                                    include_bytes!("./runtime/arrow-right.svg").to_vec()
                                )
                            ))
                            .width(Length::Units(20))
                            .on_press(Message::NextEvent)
                        ]
                        .spacing(20),
                    ]
                    .into(),
                    _ => row![column![text("Not implemented").size(50)].spacing(20)].into(),
                }
            }
        }
    }
}
