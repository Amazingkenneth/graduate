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
        ..Settings::default()
    })
}
enum Memories {
    Loading,
    Loaded(State),
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
    chosen_character: u32,
    on_image: u32,
    image: Option<image::Handle>,
    // 当前图片为 idx[on_image]
}

#[derive(Debug)]
enum Message {
    Loaded(Result<State, Error>),
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
            Command::perform(State::get_idx(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Memories::Loading => "加载中",
            Memories::Loaded(_) => "加载完毕!",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Memories::Loading => Command::none(),
            Memories::Loaded(state) => {
                match &state.stage {
                    Stage::ChoosingCharacter(chosen) => match message {
                        Message::ImageChanged(next) => {
                            let (mut chosen, state) = (chosen.clone(), state.clone());
                            let img_path = state
                                .idxtable
                                .get("image")
                                .expect("Cannot get item `image`")
                                .as_array()
                                .expect("Cannot read as an array.")[next as usize]
                                .get("path")
                                .expect("No path value in the item.")
                                .to_owned();
                            *self = Memories::Loading;
                            Command::perform(
                                async move {
                                    chosen.on_image = next;
                                    chosen.image = Some(
                                        state
                                            .get_image(img_path.to_string())
                                            .await
                                            .expect("Cannot get image."),
                                    );
                                    Ok(state)
                                },
                                Message::Loaded,
                            )
                        }
                        _ => {
                            /* *self = Memories::Loading;
                            Command::perform(, Message::ImageChanged)*/
                            Command::none()
                        }
                    },
                    _ => Command::none(),
                }
            }
        }
    }
    fn view(&self) -> Element<Message> {
        match self {
            Memories::Loading => container(
                column![
                    text("正在加载中……").size(40),
                    text("有你，才是一班。").size(20)
                ]
                .width(Length::Shrink),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into(),
            Memories::Loaded(state) => match &state.stage {
                Stage::ChoosingCharacter(chosen) => row![
                    match &chosen.image {
                        Some(handle) => Element::from(image::viewer(handle.clone())),
                        None => Element::from(text("Not able to load image.").size(40)),
                    },
                    Self::show_name("Class 1")
                ]
                .into(),
                _ => row![Self::show_name("Not implemented")].into(),
            },
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
