#![allow(dead_code, unused_imports)]
mod audio;
mod choosing;
mod entries;
mod subscriptions;
mod visiting;
use iced::widget::{
    self, column, container, horizontal_space, image, row, scrollable, text, text_input,
    vertical_space, Column, Row,
};
use iced::{
    alignment, subscription, window, Alignment, Application, Color, Command, Element, Event,
    Length, Settings, Theme,
};
use rand::Rng;
use reqwest::Client;
use toml::value::Table;

pub fn main() -> iced::Result {
    Memories::run(Settings {
        window: window::Settings {
            size: (1400, 800),
            ..window::Settings::default()
        },
        default_font: Some(include_bytes!("./YEFONTFuJiYaTi-3.ttf")),
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
    storage: String,
    scale_factor: f64,
    theme: Theme,
    from_date: toml::value::Datetime,
}

#[derive(Clone, Debug)]
enum Stage {
    EntryEvents(EntryState),
    ChoosingCharacter(ChoosingState),
    ShowingPlots(VisitingState),
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
    on_character: Option<usize>,
    profiles: Vec<choosing::Profile>,
    avatars: Vec<choosing::Avatar>,
    description: String,
    previous_stage: EntryState,
    element_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VisitingState {
    on_character: String,
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
    BackStage,
    NextStage,
    DescriptionEdited(String),
    FinishedTyping,
    ChoseCharacter(usize),
    UnChoose,
    ChangeEmoji(usize),
    PreviousPerson,
    NextPerson,
    ScaleDown,
    ScaleEnlarge,
    ScaleRestore,
    SwapTheme,
    OpenInExplorer,
    Refresh,
}

#[derive(Clone, Debug)]
pub enum Error {
    APIError,
    JoinError,
    ParseError,
}
use tokio::task::JoinError;
impl From<JoinError> for Error {
    fn from(_: JoinError) -> Error {
        crate::Error::JoinError
    }
}

impl Application for Memories {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Memories, Command<Message>) {
        (
            Memories::Loading,
            Command::perform(State::get_idx(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            Memories::Loading => "加载中",
            Memories::Loaded(state) => match &state.stage {
                Stage::EntryEvents(_) => "让我们从这里开始吧",
                Stage::ChoosingCharacter(choosing) => match choosing.on_character {
                    None => "选取角色",
                    Some(chosen) => {
                        return format!(
                            "跟着 {}，一起来回忆这段美好时光！",
                            choosing.avatars[chosen].name.as_str().to_owned()
                        )
                    }
                },
                Stage::ShowingPlots(on_plot) => {
                    return format!("瞧瞧 {} 的这段经历", on_plot.on_character)
                }
                Stage::Graduated => "就这样，我们毕业啦",
            },
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        println!("On update()");
        match self {
            Memories::Loading => match message {
                Message::Loaded(Ok(state)) => {
                    *self = Memories::Loaded(state);
                    Command::none()
                }
                _ => Command::none(),
            },
            Memories::Loaded(state) => {
                println!("Loaded...");

                match message {
                    Message::ScaleDown => {
                        state.scale_factor /= 1.05;
                        return Command::none();
                    }
                    Message::ScaleEnlarge => {
                        state.scale_factor *= 1.05;
                        return Command::none();
                    }
                    Message::ScaleRestore => {
                        state.scale_factor = 1.0;
                        return Command::none();
                    }
                    Message::SwapTheme => {
                        state.theme = match state.theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light | Theme::Custom(_) => Theme::Dark,
                        };
                        return Command::none();
                    }
                    Message::OpenInExplorer => {
                        let mut is_file = true;
                        let filename = match &state.stage {
                            Stage::EntryEvents(ref chosen) => format!(
                                "{}{}",
                                state.storage,
                                state
                                    .get_current_event(chosen.on_event)
                                    .get("image")
                                    .expect("cannot parse `image` into an array.")
                                    .as_array()
                                    .expect("Cannot read the paths")[chosen.on_image]
                                    .as_str()
                                    .expect("Cannot convert it into String")
                                    .to_string()
                            ),
                            Stage::ChoosingCharacter(choosing) => match choosing.on_character {
                                Some(chosen) => {
                                    format!("{}/profile/{}.toml", state.storage, chosen)
                                }
                                None => {
                                    is_file = false;
                                    format!("{}/image/known_people", state.storage)
                                }
                            },
                            _ => "".to_string(),
                        };
                        if cfg!(target_os = "windows") {
                            if is_file {
                                std::process::Command::new("cmd")
                                    .args(["/C", &filename])
                                    .output()
                                    .expect("failed to execute process");
                            } else {
                                println!("filename: {}", &filename);
                                std::process::Command::new("explorer")
                                    .arg(&filename.replace("/", "\\"))
                                    .output()
                                    .expect("failed to execute process");
                            }
                        } else if cfg!(target_os = "macos") {
                            std::process::Command::new("open")
                                .arg(&filename)
                                .output()
                                .expect("failed to execute process.");
                        } else {
                            std::process::Command::new("eog")
                                .arg(&filename)
                                .output()
                                .expect("failed to execute process");
                        }
                        return Command::none();
                    }
                    _ => (),
                }
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
                            Message::NextStage => {
                                let cur_event = chosen.on_event;
                                state.from_date = state
                                    .get_current_event(cur_event)
                                    .get("date")
                                    .expect("No date value in the item.")
                                    .as_datetime()
                                    .expect("cannot convert into datetime")
                                    .to_owned();
                                let state = state.clone();
                                *self = Memories::Loading;
                                return Command::perform(
                                    choosing::get_configs(state),
                                    Message::Loaded,
                                );
                            }
                            _ => {}
                        }
                        Command::none()
                    }
                    Stage::ChoosingCharacter(ref mut choosing) => {
                        match choosing.on_character {
                            None => match message {
                                Message::DescriptionEdited(new_description) => {
                                    choosing.description = new_description;
                                    for avatar in &mut choosing.avatars {
                                        if avatar.name.contains(choosing.description.as_str()) {
                                            avatar.shown = true;
                                        } else {
                                            avatar.shown = false;
                                        }
                                    }
                                    choosing.avatars[0].shown = false;
                                }
                                Message::FinishedTyping => {
                                    for (index, avatar) in choosing.avatars.iter().enumerate() {
                                        if avatar.name.contains(choosing.description.as_str()) {
                                            choosing.on_character = Some(index);
                                            return Command::none();
                                        }
                                    }
                                }
                                Message::ChoseCharacter(chosen) => {
                                    choosing.on_character = Some(chosen);
                                    return scrollable::snap_to(
                                        choosing::generate_id(chosen),
                                        scrollable::RelativeOffset::START,
                                    );
                                }
                                Message::BackStage => {
                                    state.stage =
                                        Stage::EntryEvents(choosing.previous_stage.to_owned());
                                }
                                Message::Refresh => {
                                    let mut rng = rand::thread_rng();
                                    choosing.element_count = rng.gen_range(6..=8);
                                }
                                _ => {}
                            },
                            Some(chosen) => match message {
                                Message::UnChoose => {
                                    choosing.on_character = None;
                                }
                                Message::NextStage => {
                                    let state = state.clone();
                                    *self = Memories::Loading;
                                    return Command::perform(
                                        visiting::get_queue(state),
                                        Message::Loaded,
                                    );
                                }
                                Message::NextPerson => {
                                    choosing.on_character = Some((chosen) % choosing.avatars.len());
                                }
                                Message::PreviousPerson => {
                                    choosing.on_character = Some(
                                        (chosen + choosing.avatars.len() - 1)
                                            % choosing.avatars.len(),
                                    );
                                }
                                Message::ChoseCharacter(chosen) => {
                                    choosing.on_character = Some(chosen);
                                    return scrollable::snap_to(
                                        choosing::generate_id(chosen),
                                        scrollable::RelativeOffset::START,
                                    );
                                }
                                _ => {}
                            },
                        }
                        Command::none()
                    }
                    _ => {
                        println!("On other stages.");
                        Command::none()
                    }
                }
            }
        }
    }
    fn view(&self) -> Element<Message> {
        match self {
            Memories::Loading => {
                println!("On Memories::Loading");
                container(
                    column![
                        text("正在加载中  Loading...").size(60),
                        text("有你，才是一班。").size(30)
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
                        image::viewer(
                            chosen.preload[chosen.on_event as usize][chosen.on_image].clone()
                        )
                        .width(Length::Units(1400)),
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
                            widget::Button::new(text("从这里开始！").size(35))
                                .padding(15)
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
                                    .expect("Cannot read the paths.")
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
                            ],
                            widget::Button::new(
                                column![text("切换主题").size(30), text("Ctrl + T").size(20)]
                                    .align_items(Alignment::Center)
                                    .spacing(15)
                            )
                            .padding(10)
                            .style(iced::theme::Button::Secondary)
                            .on_press(Message::SwapTheme),
                            widget::Button::new(
                                column![text("打开对应文件").size(30), text("Ctrl + O").size(20)]
                                    .align_items(Alignment::Center)
                                    .spacing(15)
                            )
                            .padding(10)
                            .style(iced::theme::Button::Text)
                            .on_press(Message::OpenInExplorer),
                        ]
                        .spacing(20)
                        .align_items(Alignment::Center)
                        .width(Length::Fill),
                    ]
                    .align_items(Alignment::Center)
                    .into(),
                    Stage::ChoosingCharacter(choosing) => {
                        match choosing.on_character {
                            None => {
                                // let apply_button = row![,
                                //     horizontal_space(Length::Units(20))
                                // ];
                                let searchbox = row![
                                    widget::Button::new(text("返回").size(28))
                                        .style(iced::theme::Button::Secondary)
                                        .on_press(Message::BackStage)
                                        .padding(15),
                                    // .width(Length::FillPortion(150)),
                                    // horizontal_space(Length::FillPortion(41)),
                                    text_input(
                                        "输入以搜索",
                                        &choosing.description,
                                        Message::DescriptionEdited,
                                    )
                                    .size(28)
                                    .padding(15)
                                    .on_submit(Message::FinishedTyping),
                                    // .width(Length::FillPortion(618)),
                                    // horizontal_space(Length::FillPortion(191)),
                                ];
                                let mut heads = vec![vec![]];
                                let mut containing: usize = choosing.element_count;
                                for (i, avatar) in choosing.avatars.iter().enumerate() {
                                    if !avatar.shown {
                                        continue;
                                    }
                                    let photo = avatar.photo.to_owned();
                                    let viewer =
                                        widget::image(photo.clone()).height(Length::Units(200));
                                    if containing == 0 {
                                        containing = choosing.element_count;
                                        heads.push(vec![]);
                                    }
                                    containing -= 1;
                                    heads.last_mut().expect("Cannot get the last value.").push(
                                        container(
                                            widget::Button::new(
                                                column![
                                                    viewer,
                                                    text(choosing.avatars[i].name.to_owned())
                                                        .size(30)
                                                ]
                                                .align_items(Alignment::Center),
                                            )
                                            .style(iced::theme::Button::Text)
                                            .padding(10)
                                            .on_press(Message::ChoseCharacter(i)),
                                        )
                                        .width(Length::FillPortion(1))
                                        .center_x()
                                        .center_y(),
                                    );
                                }
                                let mut scroll_head = column![].align_items(Alignment::Center);
                                for it in heads {
                                    let mut cur_row = row![].spacing(5);
                                    for j in it {
                                        cur_row = cur_row.push(j);
                                    }
                                    scroll_head = scroll_head.push(cur_row);
                                }
                                let content = scrollable(
                                    column![searchbox, scroll_head,].spacing(10), /*.align_items(Alignment::Center)*/
                                );
                                container(content).width(Length::Fill).into()
                            }
                            Some(chosen) => {
                                let profile = choosing.profiles[chosen].clone();
                                let mut content =
                                    column![text(if let Some(name_en) = profile.name_en {
                                        format!("{} ({})", choosing.avatars[chosen].name, name_en)
                                    } else {
                                        choosing.avatars[chosen].name.clone()
                                    })
                                    .size(50)];
                                let apply_button = row![
                                    widget::Button::new(text("返回").size(40))
                                        .padding(15)
                                        .style(iced::theme::Button::Secondary)
                                        .on_press(Message::UnChoose),
                                    widget::Button::new(text("选好啦").size(40))
                                        .padding(15)
                                        .style(iced::theme::Button::Primary)
                                        .on_press(Message::NextStage),
                                    //horizontal_space(Length::Units(20))
                                ]
                                .spacing(20);
                                content =
                                    content.push(show_profiles(profile.nickname, "ta 的昵称"));
                                if let Some(relations) = profile.relationship {
                                    let mut lists = column![];
                                    for relation in &relations {
                                        let cur_relation =
                                            relation.as_table().expect("Cannot read as a table");
                                        let with = cur_relation
                                            .get("with")
                                            .expect("Cannot get `with`")
                                            .as_integer()
                                            .expect("`with` isn't a valid integer")
                                            as usize;
                                        lists = lists.push(row![
                                            widget::Button::new(
                                                text(choosing.avatars[with].name.clone()).size(30)
                                            )
                                            .padding(0)
                                            .on_press(Message::ChoseCharacter(with))
                                            .style(iced::theme::Button::Text),
                                            text(format!(
                                                " 是 ta 的 {}；",
                                                cur_relation.get("is").expect("Cannot get `is`")
                                            ))
                                            .size(30)
                                        ]);
                                    }
                                    if !relations.is_empty() {
                                        content = content.push(column![
                                            text("ta 的人物关系").size(50),
                                            row![horizontal_space(Length::Units(20)), lists,]
                                        ]);
                                    }
                                }
                                content = content.push(show_profiles(profile.plots, "ta 的小日常"));
                                if let Some(intro) = profile.introduction {
                                    content = content.push(column![
                                        text("ta 的自传").size(50),
                                        row![
                                            horizontal_space(Length::Units(20)),
                                            text(intro).size(30)
                                        ],
                                    ])
                                }
                                if let Some(comments) = profile.comment {
                                    let mut lists = column![].spacing(15);
                                    for comment in &comments {
                                        let cur_comment =
                                            comment.as_table().expect("Cannot read as table");
                                        let with = cur_comment
                                            .get("from")
                                            .expect("Cannot get `from`")
                                            .as_integer()
                                            .expect("`from` isn't a valid integer")
                                            as usize;
                                        lists = lists.push(column![
                                            row![
                                                text("来自 ").size(40),
                                                widget::Button::new(
                                                    text(choosing.avatars[with].name.clone())
                                                        .size(40)
                                                )
                                                .padding(0)
                                                .style(iced::theme::Button::Text)
                                                .on_press(Message::ChoseCharacter(with)),
                                                text("：").size(40)
                                            ],
                                            row![
                                                widget::Svg::new(widget::svg::Handle::from_memory(
                                                    include_bytes!("./runtime/quote-left.svg")
                                                        .to_vec()
                                                ))
                                                .width(Length::Units(30)),
                                                column![text(
                                                cur_comment
                                                    .get("description")
                                                    .expect("Cannot get `description`")
                                                    .as_str()
                                                    .expect(
                                                        "Cannot convert `description` into String"
                                                    )
                                            )
                                            .size(30),
                                            text(format!(
                                                "于 {} ",
                                                cur_comment.get("date").expect("Cannot get `date`")
                                            ))
                                            .size(30)]
                                                .align_items(Alignment::End)
                                            ]
                                        ]);
                                    }
                                    if !comments.is_empty() {
                                        content = content.push(column![
                                            text("大家对 ta 的评价").size(50),
                                            row![horizontal_space(Length::Units(20)), lists,]
                                        ]);
                                    }
                                }
                                container(
                                    scrollable(
                                        column![content.spacing(5), apply_button]
                                            .align_items(Alignment::End),
                                    )
                                    .id(choosing::generate_id(chosen)),
                                )
                                //.width(Length::Fill)
                                .center_x()
                                .center_y()
                                .into()
                            }
                        }
                    }
                    _ => {
                        println!("current state: {:?}", state);
                        row![column![text("Not implemented").size(50)].spacing(20)].into()
                    }
                }
            }
        }
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        // from iced_native::events_with
        match self {
            Memories::Loading => iced::Subscription::none(),
            Memories::Loaded(state) => match state.stage {
                Stage::EntryEvents(_) => {
                    iced::subscription::events_with(subscriptions::on_entry_state)
                }
                Stage::ChoosingCharacter(_) => {
                    iced::subscription::events_with(subscriptions::on_choosing_character)
                }
                _ => iced::Subscription::none(),
            },
        }
    }
    fn scale_factor(&self) -> f64 {
        match self {
            Memories::Loading => 1.0,
            Memories::Loaded(state) => state.scale_factor,
        }
    }

    fn theme(&self) -> Theme {
        match self {
            Memories::Loading => Theme::Light,
            Memories::Loaded(state) => state.theme.clone(),
        }
    }
}

pub fn button_from_svg(position: Vec<u8>) -> widget::Button<'static, Message> {
    widget::Button::new(widget::Svg::new(widget::svg::Handle::from_memory(position)))
        .style(iced::theme::Button::Text)
}

fn show_profiles(item: Option<toml::value::Array>, with_name: &str) -> Element<Message> {
    if let Some(item) = item {
        let mut lists = column![];
        for i in &item {
            lists = lists.push(text(i.as_str().expect("Not valid String").to_string()).size(40));
        }
        if !item.is_empty() {
            return Element::from(column![
                text(with_name).size(50),
                row![horizontal_space(Length::Units(20)), lists,]
            ]);
        }
    }
    Element::from(column![])
}
