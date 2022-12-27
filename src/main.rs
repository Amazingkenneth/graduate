#![allow(dead_code, unused_imports)]
mod exchange;
use iced::widget::{
    self, column, container, horizontal_space, image, row, text, vertical_space, Column,
};
use iced::{
    keyboard, subscription, window, Application, Color, Command, Element, Event, Length, Settings,
    Theme,
};
use reqwest::Client;
use toml::value::Table;

macro_rules! with_key {
    ($key: path) => {
        keyboard::Event::KeyPressed {
            key_code: $key,
            modifiers: _,
        }
    };
}
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
    EntryEvents(EntryState),
    ChoosingCharacter(ChoosingState),
    ShowingPlots,
    Graduated,
}

#[derive(Clone, Debug, Default)]
pub struct EntryState {
    on_event: usize,
    on_image: usize,
    preload: Vec<Vec<image::Handle>>,
    // 当前图片为 preload[on_event][on_image]
}

#[derive(Clone, Debug, Default)]
pub struct ChoosingState {
    on_character: Option<u32>,
    image: Option<image::Handle>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Loaded(Result<State, Error>),
    FetchImage(Result<State, Error>),
    LoadedImage(Result<EntryState, Error>),
    PreviousEvent,
    NextEvent,
    PreviousPhoto,
    NextPhoto,
    NextStage,
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
            Command::perform(State::get_idx(), Message::Loaded),
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
                /*Message::FetchImage(Ok(state)) => {
                    // *self = Memories::Loading;
                    // let
                    Command::perform(exchange::change_image(state, 0, 0), Message::Loaded)
                }*/
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
                    Stage::EntryEvents(ref mut chosen) => {
                        match message {
                            Message::PreviousEvent => {
                                chosen.on_event = (chosen.on_event + chosen.preload.len() - 1)
                                    % chosen.preload.len();
                                chosen.on_image = 0;
                            }
                            Message::NextEvent => {
                                chosen.on_event = (chosen.on_event + 1) % chosen.preload.len();
                                chosen.on_image = 0;
                            }
                            Message::PreviousPhoto => {
                                chosen.on_image =
                                    (chosen.on_image + chosen.preload[chosen.on_event].len() - 1)
                                        % chosen.preload[chosen.on_event].len();
                            }
                            Message::NextPhoto => {
                                chosen.on_image =
                                    (chosen.on_image + 1) % chosen.preload[chosen.on_event].len();
                            }
                            Message::Loaded(_) => {
                                println!("On Message::Loaded");
                            }
                            Message::NextStage => {
                                state.stage = Stage::ChoosingCharacter(Default::default());
                            }
                            _ => {
                                println!("Not regular message");
                            }
                        }
                        Command::none()
                    }
                    _ => {
                        println!("Some other message.");
                        Command::none()
                    }
                }
            }
        }
    }
    fn view(&self) -> Element<Message> {
        //println!("On view()... self.stage = {:?}", self.stage);
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
                    Stage::EntryEvents(chosen) => row![
                        Element::from(image::viewer(
                            chosen.preload[chosen.on_event as usize][chosen.on_image].clone()
                        )),
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
                            text(format!(
                                "拍摄于 {}",
                                state
                                    .get_current_event(chosen.on_event)
                                    .get("date")
                                    .expect("No date value in the item.")
                                    .as_datetime()
                                    .expect("cannot convert into datetime")
                                    .date
                                    .as_ref()
                                    .expect("Cannot get its date")
                            ))
                            .size(30),
                            widget::Button::new(text("从这里开始！").size(25))
                                .padding(20)
                                .style(iced::theme::Button::Positive)
                                .on_press(Message::NextStage),
                            row![
                                button_from_svg(
                                    include_bytes!("./runtime/arrow-left.svg").to_vec()
                                )
                                .width(Length::Units(80))
                                .on_press(Message::PreviousEvent),
                                if state
                                    .get_current_event(chosen.on_event)
                                    .get("image")
                                    .expect("No image value in the item.")
                                    .as_array()
                                    .expect("Cannot read the path.")
                                    .len()
                                    > 1
                                {
                                    Element::from(column![
                                        button_from_svg(
                                            include_bytes!("./runtime/up.svg").to_vec()
                                        )
                                        .width(Length::Units(40))
                                        .on_press(Message::PreviousPhoto),
                                        button_from_svg(
                                            include_bytes!("./runtime/down.svg").to_vec()
                                        )
                                        .width(Length::Units(40))
                                        .on_press(Message::NextPhoto)
                                    ])
                                } else {
                                    Element::from(horizontal_space(Length::Units(40)))
                                },
                                button_from_svg(
                                    include_bytes!("./runtime/arrow-right.svg").to_vec()
                                )
                                .width(Length::Units(80))
                                .on_press(Message::NextEvent),
                            ]
                        ]
                        .spacing(20),
                    ]
                    .into(),
                    _ => row![column![text("Not implemented").size(50)].spacing(20)].into(),
                }
            }
        }
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        use keyboard::KeyCode;
        match self {
            Memories::Loading => iced::Subscription::none(),
            Memories::Loaded(state) => match state.stage {
                Stage::EntryEvents(_) => subscription::events_with(|event, _status| match event {
                    Event::Keyboard(keyboard_event) => match keyboard_event {
                        with_key!(KeyCode::Left) => Some(Message::PreviousEvent),
                        with_key!(KeyCode::Right) => Some(Message::NextEvent),
                        with_key!(keyboard::KeyCode::Up) => Some(Message::PreviousPhoto),
                        with_key!(keyboard::KeyCode::Down) => Some(Message::NextPhoto),
                        keyboard::Event::KeyPressed {
                            key_code: keyboard::KeyCode::Tab,
                            modifiers,
                        } => Some(if modifiers.shift() {
                            Message::PreviousEvent
                        } else {
                            Message::NextEvent
                        }),
                        with_key!(keyboard::KeyCode::Space) => Some(Message::NextEvent),
                        with_key!(keyboard::KeyCode::Enter) => Some(Message::NextStage),
                        _ => None,
                    },
                    _ => None,
                }),
                _ => iced::Subscription::none(),
            },
        }
    }
}

pub fn button_from_svg(position: Vec<u8>) -> widget::Button<'static, Message> {
    widget::Button::new(widget::Svg::new(widget::svg::Handle::from_memory(position)))
        .style(iced::theme::Button::Text)
}
