#![allow(dead_code)]
mod audio;
mod choosing;
mod configs;
mod entries;
mod graduation;
mod imageviewer;
mod overlay;
mod pinpoint;
mod quadbutton;
mod sink;
mod subscriptions;
mod visiting;

use crate::overlay::Offset;
use configs::Configs;
use iced::widget::{
    self, column, container, horizontal_space, image, row, scrollable, text, text_input,
    vertical_space,
};
use iced::window::Mode;
use iced::{window, Alignment, Application, Color, Command, Element, Length, Settings, Theme};
use rand::Rng;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use toml::value::Table;

pub static DELETE_FILES_ON_EXIT: AtomicBool = AtomicBool::new(false);
pub static SCALE_FACTOR: AtomicU64 = AtomicU64::new(0x3FF0000000000000); // 1.0f64

pub fn store_scale_factor(value: f64) {
    let as_u64 = value.to_bits();
    SCALE_FACTOR.store(as_u64, Ordering::Relaxed);
}
pub fn load_scale_factor() -> f64 {
    f64::from_bits(SCALE_FACTOR.load(Ordering::Relaxed))
}

fn main() {
    #[cfg(target_os = "macos")]
    let specific = iced::window::PlatformSpecific {
        title_hidden: true,
        titlebar_transparent: true,
        fullsize_content_view: true,
    };

    #[cfg(not(target_os = "macos"))]
    let specific = Default::default();

    Memories::run(Settings {
        window: window::Settings {
            platform_specific: specific,
            size: (1500, 900),
            icon: Some(
                iced::window::icon::from_file_data(
                    include_bytes!("./runtime/icon.png"),
                    Some(::image::ImageFormat::Png),
                )
                .unwrap(),
            ),
            ..window::Settings::default()
        },
        default_font: iced::Font::with_name("YEFONTFuJiYaTi"),
        ..Settings::default()
    })
    .unwrap();
    if DELETE_FILES_ON_EXIT.load(Ordering::SeqCst) {
        let proj_dir = directories::ProjectDirs::from("", "Class1", "Graduate").unwrap();
        std::fs::remove_dir_all(proj_dir.data_dir()).unwrap();
        std::fs::remove_dir_all(proj_dir.config_dir()).unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum Memories {
    Initialization,
    Loading(Configs), // 有加载任务尚未完成
    Loaded(State),    // 已完成加载，等待下个事件
}

#[derive(Clone, Debug)]
pub struct State {
    stage: Stage,
    idxtable: Table,
    storage: String,
    configs: configs::Configs,
}

#[derive(Clone, Debug)]
enum Stage {
    EntryEvents(EntryState),
    ChoosingCharacter(ChoosingState),
    ShowingPlots(VisitingState),
    Graduated(GraduationState),
}

#[derive(Clone, Debug, Default)]
pub struct EntryState {
    on_event: usize,
    on_image: usize,
    preload: Arc<Mutex<Vec<Vec<image::Handle>>>>,
    // 当前图片为 preload[on_event][on_image]
}

#[derive(Clone, Debug, Default)]
pub struct ChoosingState {
    on_character: Option<usize>,
    profiles: Vec<choosing::Profile>,
    avatars: Vec<choosing::Avatar>,
    homepage_offset: scrollable::RelativeOffset,
    description: String,
    previous_stage: Option<EntryState>,
    element_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VisitingState {
    character_name: String,
    on_event: usize,
    events: Arc<Mutex<Vec<visiting::Event>>>,
    homepage_offset: scrollable::RelativeOffset,
}

#[derive(Clone, Debug, Default)]
pub struct GraduationState {
    homepage_offset: scrollable::RelativeOffset,
    show_panel: bool,
    on_image: usize,
    images: Vec<graduation::Panorama>,
}

#[derive(Clone, Debug)]
pub enum Message {
    BackStage,
    FontLoaded(Result<(), iced::font::Error>),
    ChoseCharacter(usize),
    ClickedPin(usize),
    CopyText(String),
    DescriptionEdited(String),
    EscapeFullScreen,
    FetchImage(Result<Memories, Error>),
    FinishedTyping,
    HideSettings,
    HomepageScrolled(scrollable::Viewport),
    IsDarkTheme(bool),
    Loaded(Result<State, Error>),
    LoadedImage(Result<EntryState, Error>),
    ModifyVolume(f32),
    NextEvent,
    NextPerson,
    NextPhoto,
    NextSong,
    NextStage,
    OpenUrl(Option<String>),
    OpenSettings,
    PreviousEvent,
    PreviousPerson,
    PreviousPhoto,
    Refresh,
    ScaleDown,
    ScaleEnlarge,
    ScaleRestore,
    SelectedImage(String),
    SwitchDeleteFilesStatus,
    SwitchMusicStatus,
    ToggleMode,
    TogglePanelShown,
    UnChoose,
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
            Memories::Initialization,
            Command::batch(vec![
                iced::font::load(include_bytes!("./YEFONTFuJiYaTi.ttf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(State::get_idx(None), Message::Loaded),
            ]),
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
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
                    return format!("瞧瞧 {} 的这段经历", on_plot.character_name)
                }
                Stage::Graduated(img) => {
                    return format!(
                        "来看看 {} 吧",
                        img.images[graduation::ON_LOCATION.load(Ordering::Relaxed)].image_names
                            [img.on_image]
                    )
                }
            },
            _ => "加载中",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Memories::Initialization => {
                match message {
                    Message::Loaded(Ok(mut state)) => {
                        configs::save_configs(&mut state);
                        *self = Memories::Loaded(state);
                    }
                    Message::FetchImage(Ok(memo)) => {
                        *self = memo;
                    }
                    Message::ScaleDown => {
                        store_scale_factor(load_scale_factor() / 1.05);
                    }
                    Message::ScaleEnlarge => {
                        store_scale_factor(load_scale_factor() * 1.05);
                    }
                    Message::ScaleRestore => {
                        store_scale_factor(1.0);
                    }
                    _ => (),
                }
                Command::none()
            }
            Memories::Loading(config) => {
                match message {
                    // 记得要把这里的代码复制到 `Memories::Loaded(_)` 里面噢
                    Message::ToggleMode => {
                        let mode = if config.full_screened {
                            Mode::Windowed
                        } else {
                            Mode::Fullscreen
                        };
                        config.full_screened ^= true;
                        return iced::window::change_mode(mode);
                    }
                    Message::EscapeFullScreen => {
                        config.full_screened = false;
                        return iced::window::change_mode(Mode::Windowed);
                    }
                    Message::Loaded(Ok(mut state)) => {
                        configs::save_configs(&mut state);
                        *self = Memories::Loaded(state);
                    }
                    Message::FetchImage(Ok(memo)) => {
                        *self = memo;
                    }
                    Message::ScaleDown => {
                        store_scale_factor(load_scale_factor() / 1.05);
                    }
                    Message::ScaleEnlarge => {
                        store_scale_factor(load_scale_factor() * 1.05);
                    }
                    Message::ScaleRestore => {
                        store_scale_factor(1.0);
                    }
                    Message::IsDarkTheme(is_dark) => {
                        if is_dark {
                            config.theme = Theme::Dark;
                        } else {
                            config.theme = Theme::Light;
                        }
                    }
                    Message::OpenSettings => {
                        config.shown = true;
                    }
                    Message::HideSettings => {
                        config.shown = false;
                    }
                    Message::SwitchDeleteFilesStatus => {
                        DELETE_FILES_ON_EXIT.fetch_xor(true, Ordering::Relaxed);
                    }
                    Message::SwitchMusicStatus => {
                        let audio_stream = audio::AUDIO_PLAYER.lock().unwrap();
                        let sink = &audio_stream.as_ref().unwrap().sink;
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                        return Command::none();
                    }
                    Message::NextSong => {
                        audio::AUDIO_PLAYER
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .sink
                            .stop();
                        return Command::none();
                    }
                    Message::ModifyVolume(new_volume) => {
                        config.volume_percentage = new_volume;
                        audio::AUDIO_PLAYER
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .sink
                            .set_volume(new_volume / 100.0);
                        return Command::none();
                    }
                    _ => (),
                }
                Command::none()
            }
            Memories::Loaded(state) => {
                match message {
                    // 记得要把这里的代码复制到 `Memories::Loading(_)` 里面噢
                    Message::ToggleMode => {
                        let mode = if state.configs.full_screened {
                            Mode::Windowed
                        } else {
                            Mode::Fullscreen
                        };
                        state.configs.full_screened ^= true;
                        return iced::window::change_mode(mode);
                    }
                    Message::EscapeFullScreen => {
                        state.configs.full_screened = false;
                        return iced::window::change_mode(Mode::Windowed);
                    }
                    Message::ScaleDown => {
                        store_scale_factor(load_scale_factor() / 1.05);
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::ScaleEnlarge => {
                        store_scale_factor(load_scale_factor() * 1.05);
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::ScaleRestore => {
                        store_scale_factor(1.0);
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::IsDarkTheme(is_dark) => {
                        if is_dark {
                            state.configs.theme = Theme::Dark;
                        } else {
                            state.configs.theme = Theme::Light;
                        }
                        return Command::none();
                    }
                    Message::OpenSettings => {
                        state.configs.shown = true;
                        return Command::none();
                    }
                    Message::HideSettings => {
                        state.configs.shown = false;
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::SwitchDeleteFilesStatus => {
                        DELETE_FILES_ON_EXIT.fetch_xor(true, Ordering::Relaxed);
                        return Command::none();
                    }
                    Message::SwitchMusicStatus => {
                        let audio_stream = audio::AUDIO_PLAYER.lock().unwrap();
                        let sink = &audio_stream.as_ref().unwrap().sink;
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                        return Command::none();
                    }
                    Message::NextSong => {
                        audio::AUDIO_PLAYER
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .sink
                            .stop();
                        return Command::none();
                    }
                    Message::ModifyVolume(new_volume) => {
                        state.configs.volume_percentage = new_volume;
                        audio::AUDIO_PLAYER
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .sink
                            .set_volume(new_volume / 100.0);
                        return Command::none();
                    }
                    Message::OpenUrl(filename) => {
                        if let Some(name) = filename {
                            subscriptions::open_url(name);
                        } else {
                            let name = match &state.stage {
                                Stage::EntryEvents(ref chosen) => format!(
                                    "{}{}",
                                    state.storage,
                                    state
                                        .get_current_event(chosen.on_event)
                                        .get("image")
                                        .unwrap()
                                        .as_array()
                                        .unwrap()[chosen.on_image]
                                        .as_str()
                                        .unwrap()
                                        .to_string()
                                ),
                                Stage::ChoosingCharacter(choosing) => match choosing.on_character {
                                    Some(chosen) => {
                                        format!("{}/profile/{}.toml", state.storage, chosen)
                                    }
                                    None => {
                                        format!("{}/image/known_people", state.storage)
                                    }
                                },
                                Stage::ShowingPlots(displayer) => {
                                    let events = displayer.events.lock().unwrap();
                                    let on_image = &events[displayer.on_event].on_experience;
                                    format!(
                                        "{}{}",
                                        state.storage,
                                        events[displayer.on_event].experiences[*on_image].path
                                    )
                                }
                                Stage::Graduated(vision) => {
                                    format!(
                                        "{}/image/panorama/{}.jpg",
                                        state.storage,
                                        vision.images
                                            [graduation::ON_LOCATION.load(Ordering::Relaxed)]
                                        .image_names[vision.on_image]
                                    )
                                }
                            };
                            subscriptions::open_url(name);
                        }
                        return Command::none();
                    }
                    Message::BackStage | Message::NextStage => {
                        configs::save_configs(state);
                        // 这里不可以直接返回！
                    }
                    _ => (),
                }
                match state.stage {
                    Stage::EntryEvents(ref mut chosen) => {
                        match message {
                            Message::PreviousEvent => {
                                let len = chosen.preload.lock().unwrap().len();
                                chosen.on_event = (chosen.on_event + len - 1) % len;
                                chosen.on_image = 0;
                            }
                            Message::NextEvent => {
                                chosen.on_event =
                                    (chosen.on_event + 1) % chosen.preload.lock().unwrap().len();
                                chosen.on_image = 0;
                            }
                            Message::PreviousPhoto => {
                                let preload = chosen.preload.lock().unwrap();
                                chosen.on_image =
                                    (chosen.on_image + preload[chosen.on_event].len() - 1)
                                        % preload[chosen.on_event].len();
                            }
                            Message::NextPhoto => {
                                let preload = chosen.preload.lock().unwrap();
                                chosen.on_image =
                                    (chosen.on_image + 1) % preload[chosen.on_event].len();
                            }
                            Message::NextStage => {
                                let cur_event = chosen.on_event;
                                state.configs.from_date = state
                                    .get_current_event(cur_event)
                                    .get("date")
                                    .unwrap()
                                    .as_datetime()
                                    .unwrap()
                                    .into();
                                let state = state.to_owned();
                                *self = Memories::Loading(state.configs.clone());
                                return Command::perform(
                                    choosing::get_configs(
                                        None,
                                        scrollable::RelativeOffset::START,
                                        state,
                                    ),
                                    Message::Loaded,
                                );
                            }
                            _ => {}
                        }
                        return imageviewer::reset_scale(imageviewer::entryevents_viewer_id(
                            chosen.on_event,
                            chosen.on_image,
                        ));
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
                                        choosing::generate_scrollable_id(chosen),
                                        scrollable::RelativeOffset::START,
                                    );
                                }
                                Message::BackStage => {
                                    if let Some(previous) = choosing.previous_stage.to_owned() {
                                        state.stage = Stage::EntryEvents(previous);
                                    } else {
                                        let state = state.to_owned();
                                        *self = Memories::Loading(state.configs.clone());
                                        return Command::perform(
                                            State::get_idx(Some(state)),
                                            Message::Loaded,
                                        );
                                    }
                                }
                                Message::Refresh => {
                                    let mut rng = rand::thread_rng();
                                    choosing.element_count = rng.gen_range(6..=8);
                                }
                                Message::HomepageScrolled(new_offset) => {
                                    choosing.homepage_offset = new_offset.relative_offset();
                                }
                                _ => {}
                            },
                            Some(chosen) => match message {
                                Message::UnChoose => {
                                    choosing.on_character = None;
                                    return scrollable::snap_to(
                                        scrollable::Id::new("HomepageScrollable"),
                                        choosing.homepage_offset,
                                    );
                                }
                                Message::NextStage => {
                                    let state = state.to_owned();
                                    *self = Memories::Loading(state.configs.clone());
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
                                        choosing::generate_scrollable_id(chosen),
                                        scrollable::RelativeOffset::START,
                                    );
                                }
                                Message::CopyText(text) => {
                                    return iced::clipboard::write(text);
                                }
                                _ => {}
                            },
                        }
                        Command::none()
                    }
                    Stage::ShowingPlots(ref mut displayer) => {
                        let mut next_stage = false;
                        let cur_image = {
                            let mut events = displayer.events.lock().unwrap();
                            let len = &events[displayer.on_event].experiences.len();
                            let on_image = &mut events[displayer.on_event].on_experience;
                            match message {
                                Message::PreviousEvent => {
                                    displayer.on_event -= 1;
                                }
                                Message::NextEvent => {
                                    if displayer.on_event + 1 == events.len() {
                                        next_stage = true;
                                    } else {
                                        displayer.on_event += 1;
                                    }
                                }
                                Message::NextStage => {
                                    next_stage = true;
                                }
                                Message::PreviousPhoto => {
                                    *on_image = (*on_image + len - 1) % len;
                                }
                                Message::NextPhoto => {
                                    *on_image = (*on_image + 1) % len;
                                }
                                _ => {}
                            }
                            (displayer.on_event, events[displayer.on_event].on_experience)
                        };
                        if next_stage {
                            let state = state.to_owned();
                            *self = Memories::Loading(state.configs.clone());
                            return Command::perform(graduation::load_map(state), Message::Loaded);
                        }
                        match message {
                            Message::BackStage => {
                                let homepage_offset = displayer.homepage_offset;
                                let state = state.to_owned();
                                *self = Memories::Loading(state.configs.clone());
                                return Command::perform(
                                    choosing::get_configs(None, homepage_offset, state),
                                    Message::Loaded,
                                );
                            }
                            Message::PreviousEvent | Message::NextEvent => {
                                let need_force_run = {
                                    displayer.events.lock().unwrap()[displayer.on_event]
                                        .get_image_handle()
                                };
                                if let None = need_force_run {
                                    let handle = {
                                        let events = displayer.events.lock().unwrap();
                                        events[displayer.on_event].get_join_handle()
                                    };
                                    let config = state.configs.clone();
                                    let memo = std::mem::replace(self, Memories::Loading(config));
                                    return Command::perform(
                                        visiting::force_load(handle, memo),
                                        Message::FetchImage,
                                    );
                                }
                                visiting::load_images(state);
                            }
                            _ => {}
                        }
                        return imageviewer::reset_scale(imageviewer::showingplots_viewer_id(
                            cur_image.0,
                            cur_image.1,
                        ));
                    }
                    Stage::Graduated(ref mut vision) => {
                        match message {
                            Message::TogglePanelShown => {
                                vision.show_panel ^= true;
                            }
                            Message::ClickedPin(new_on) => {
                                graduation::ON_LOCATION.store(new_on, Ordering::Relaxed);
                                vision.on_image = 0;
                            }
                            Message::SelectedImage(s) => {
                                for (index, value) in vision.images
                                    [graduation::ON_LOCATION.load(Ordering::Relaxed)]
                                .image_names
                                .iter()
                                .enumerate()
                                {
                                    if value == &s {
                                        vision.on_image = index;
                                        break;
                                    }
                                }
                            }
                            Message::BackStage => {
                                let homepage_offset = vision.homepage_offset;
                                let state = state.to_owned();
                                *self = Memories::Loading(state.configs.clone());
                                return Command::perform(
                                    choosing::get_configs(None, homepage_offset, state),
                                    Message::Loaded,
                                );
                            }
                            _ => {}
                        }
                        Command::none()
                    }
                }
            }
        }
    }
    fn view(&self) -> Element<Message> {
        match self {
            Memories::Loading(_) | Memories::Initialization => {
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
                let content: Element<Message, iced::Renderer> = match &state.stage {
                    Stage::EntryEvents(chosen) => row![
                        imageviewer::Viewer::new(
                            chosen.preload.lock().unwrap()[chosen.on_event][chosen.on_image]
                                .clone(),
                        )
                        .id(imageviewer::entryevents_viewer_id(
                            chosen.on_event,
                            chosen.on_image
                        ))
                        .width(Length::FillPortion(4))
                        .height(Length::Fill),
                        column![
                            widget::tooltip(
                                button_from_svg(include_bytes!("./runtime/gears.svg"))
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::OpenSettings),
                                "设置「按 E」",
                                iced::widget::tooltip::Position::Top
                            )
                            .style(iced::theme::Container::Box),
                            text(
                                state
                                    .get_current_event(chosen.on_event)
                                    .get("description")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                            )
                            .size(50),
                            text(format!(
                                "拍摄于 {}",
                                state
                                    .get_current_event(chosen.on_event)
                                    .get("date")
                                    .unwrap()
                                    .as_datetime()
                                    .unwrap()
                                    .date
                                    .as_ref()
                                    .unwrap()
                            ))
                            .size(30),
                            widget::Button::new(text("从这里开始！").size(35))
                                .padding(15)
                                // .style(iced::theme::Button::Positive)
                                .style(iced::theme::Button::Custom(Box::new(
                                    quadbutton::QuadButton::Positive
                                )))
                                .on_press(Message::NextStage),
                            row![
                                button_from_svg(include_bytes!("./runtime/arrow-left.svg"))
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::PreviousEvent),
                                if state
                                    .get_current_event(chosen.on_event)
                                    .get("image")
                                    .unwrap()
                                    .as_array()
                                    .unwrap()
                                    .len()
                                    > 1
                                {
                                    Element::from(column![
                                        button_from_svg(include_bytes!("./runtime/up.svg"))
                                            .width(Length::Fixed(40.0))
                                            .on_press(Message::PreviousPhoto),
                                        button_from_svg(include_bytes!("./runtime/down.svg"))
                                            .width(Length::Fixed(40.0))
                                            .on_press(Message::NextPhoto)
                                    ])
                                } else {
                                    Element::from(horizontal_space(Length::Fixed(40.0)))
                                },
                                button_from_svg(include_bytes!("./runtime/arrow-right.svg"))
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::NextEvent),
                            ],
                            widget::tooltip(
                                widget::Button::new(text("打开对应文件").size(30))
                                    .padding(10)
                                    .style(iced::theme::Button::Secondary)
                                    .on_press(Message::OpenUrl(None)),
                                "按 O",
                                widget::tooltip::Position::Bottom
                            )
                            .gap(15)
                            .style(iced::theme::Container::Box),
                        ]
                        .spacing(20)
                        .align_items(Alignment::Center)
                        .width(Length::FillPortion(1)),
                    ]
                    .align_items(Alignment::Center)
                    .height(Length::Fill)
                    .into(),
                    Stage::ChoosingCharacter(choosing) => match choosing.on_character {
                        None => {
                            let searchbox = row![
                                widget::Button::new(text("返回").size(28))
                                    .style(iced::theme::Button::Secondary)
                                    .on_press(Message::BackStage)
                                    .padding(15),
                                text_input("输入以搜索", &choosing.description,)
                                    .on_input(Message::DescriptionEdited)
                                    .size(28)
                                    .padding(15)
                                    .on_submit(Message::FinishedTyping),
                            ];
                            let mut heads = vec![vec![]];
                            let mut containing: usize = choosing.element_count;
                            for (i, avatar) in choosing.avatars.iter().enumerate() {
                                if !avatar.shown {
                                    continue;
                                }
                                let photo = avatar.photo.to_owned();
                                let viewer =
                                    widget::image(photo.clone()).height(Length::Fixed(200.0));
                                if containing == 0 {
                                    containing = choosing.element_count;
                                    heads.push(vec![]);
                                }
                                containing -= 1;
                                heads.last_mut().unwrap().push(
                                    container(
                                        widget::Button::new(
                                            column![
                                                viewer,
                                                text(choosing.avatars[i].name.to_owned()).size(30)
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
                            let content = scrollable(column![searchbox, scroll_head,].spacing(10))
                                .id(scrollable::Id::new("HomepageScrollable"))
                                .on_scroll(Message::HomepageScrolled);
                            container(content).width(Length::Fill).into()
                        }
                        Some(chosen) => {
                            let profile = choosing.profiles[chosen].clone();
                            let link_color = if state.configs.theme == Theme::Light {
                                Color::from_rgb8(0, 25, 175)
                            } else {
                                Color::from_rgb8(255, 215, 121)
                            };
                            let mut content =
                                column![text(if let Some(name_en) = profile.name_en {
                                    format!("{} ({})", choosing.avatars[chosen].name, name_en)
                                } else {
                                    choosing.avatars[chosen].name.clone()
                                })
                                .size(50)];
                            content = content.push(show_profiles(profile.nickname, "ta 的昵称"));
                            if let Some(relations) = profile.relationship {
                                let mut lists = column![];
                                for relation in &relations {
                                    let cur_relation = relation.as_table().unwrap();
                                    let author =
                                        cur_relation.get("by").unwrap().as_integer().unwrap()
                                            as usize;
                                    lists = lists.push(row![
                                        text("是 ").size(30),
                                        widget::Button::new(
                                            text(choosing.avatars[author].name.clone())
                                                .size(30)
                                                .style(link_color)
                                        )
                                        .padding(0)
                                        .on_press(Message::ChoseCharacter(author))
                                        .style(iced::theme::Button::Text),
                                        text(format!(
                                            " 的 {}；",
                                            cur_relation.get("is").unwrap().as_str().unwrap()
                                        ))
                                        .size(30)
                                    ]);
                                }
                                if !relations.is_empty() {
                                    content = content.push(column![
                                        text("ta 的身份").size(50),
                                        row![horizontal_space(Length::Fixed(20.0)), lists,]
                                    ]);
                                }
                            }

                            let mut emojis = row![].align_items(Alignment::Center).spacing(5);
                            for (j, i) in choosing.avatars[chosen].emoji.iter().enumerate() {
                                emojis = emojis.push(
                                    column![
                                        imageviewer::Viewer::new(i.emoji.clone())
                                            .height(Length::Fixed(400.0))
                                            .id(imageviewer::emoji_id(chosen, j)),
                                        text(i.emoji_name.clone()).size(30),
                                        vertical_space(Length::Fixed(5.0))
                                    ]
                                    .align_items(Alignment::Center),
                                );
                            }
                            let mut content = column![row![
                                content,
                                scrollable(emojis).direction(scrollable::Direction::Vertical(
                                    scrollable::Properties::new()
                                ))
                            ]];
                            content = content.push(show_profiles(profile.plots, "ta 的小日常"));
                            if let Some(summary) = profile.anecdote {
                                content = content.push(column![
                                    text("关于 ta").size(50),
                                    row![
                                        horizontal_space(Length::Fixed(20.0)),
                                        column![
                                            text(format!(
                                                "兴趣爱好：{}",
                                                summary.get("interests").unwrap().as_str().unwrap()
                                            ))
                                            .size(32)
                                            .style(Color::from_rgb8(240, 134, 80)),
                                            text(format!(
                                                "最想做的事：{}",
                                                summary.get("want").unwrap().as_str().unwrap()
                                            ))
                                            .size(32)
                                            .style(Color::from_rgb8(240, 135, 132)),
                                            text(format!(
                                                "最尴尬的事：{}",
                                                summary
                                                    .get("embarrassment")
                                                    .unwrap()
                                                    .as_str()
                                                    .unwrap()
                                            ))
                                            .size(32)
                                            .style(Color::from_rgb8(127, 130, 187))
                                        ]
                                    ],
                                ])
                            }
                            if let Some(intro) = profile.introduction {
                                content = content.push(column![
                                    text("ta 的自传").size(50),
                                    row![
                                        horizontal_space(Length::Fixed(20.0)),
                                        text(intro).size(30)
                                    ],
                                ])
                            }
                            if let Some(articles) = profile.article {
                                let mut lists = column![];
                                let mut article_vec = vec![];

                                for article in &articles {
                                    let cur_article = article.as_table().unwrap();
                                    let content =
                                        cur_article.get("content").unwrap().as_str().unwrap();
                                    let date =
                                        cur_article.get("date").unwrap().as_datetime().unwrap();
                                    let link = cur_article.get("link").unwrap().as_str().unwrap();
                                    article_vec.push((date, content, link));
                                }
                                article_vec.sort_unstable();
                                for i in article_vec {
                                    lists = lists.push(
                                        row![
                                            text(i.0)
                                                .size(20)
                                                .style(Color::from_rgb8(85, 143, 128)),
                                            widget::Button::new(
                                                text(i.1).size(30).style(link_color)
                                            )
                                            .padding(0)
                                            .on_press(Message::OpenUrl(Some(i.2.to_string())))
                                            .style(iced::theme::Button::Text),
                                            widget::tooltip(
                                                widget::button(widget::Svg::new(
                                                    widget::svg::Handle::from_memory(
                                                        include_bytes!("./runtime/clipboard.svg")
                                                            .to_vec()
                                                    )
                                                ))
                                                .height(Length::Fixed(32.0))
                                                .width(Length::Fixed(50.0))
                                                .style(iced::theme::Button::Text)
                                                .on_press(Message::CopyText(i.2.to_string())),
                                                "复制链接",
                                                widget::tooltip::Position::Right
                                            )
                                        ]
                                        .align_items(Alignment::Start),
                                    );
                                }
                                if !articles.is_empty() {
                                    content =
                                        content.push(widget::vertical_space(Length::Fixed(10.0)));
                                    content = content.push(row![
                                        widget::Svg::new(widget::svg::Handle::from_memory(
                                            include_bytes!("./runtime/link.svg").to_vec()
                                        ))
                                        .width(Length::Fixed(40.0)),
                                        horizontal_space(15.0),
                                        lists
                                    ]);
                                }
                            }
                            if let Some(reviews) = profile.reviews {
                                let mut lists = column![].spacing(15);
                                for (index, review) in reviews.iter().enumerate() {
                                    let cur_review = review.as_str().unwrap();
                                    lists = lists.push(column![
                                        text(choosing::SEMESTER_NAMES[index])
                                            .size(25)
                                            .style(Color::from_rgb8(120, 158, 204)),
                                        row![
                                            horizontal_space(Length::Fixed(20.0)),
                                            text(cur_review).size(30)
                                        ]
                                    ]);
                                }
                                content = content.push(column![
                                    text("班主任评语").size(50),
                                    row![horizontal_space(Length::Fixed(20.0)), lists,]
                                ]);
                            }
                            if let Some(comments) = profile.comment {
                                let mut lists = column![].spacing(15);
                                for comment in &comments {
                                    let cur_comment = comment.as_table().unwrap();
                                    let with =
                                        cur_comment.get("from").unwrap().as_integer().unwrap()
                                            as usize;
                                    lists = lists.push(column![
                                        row![
                                            text("来自 ").size(40),
                                            widget::Button::new(
                                                text(choosing.avatars[with].name.clone()).size(40).style(link_color)
                                            )
                                            .padding(0)
                                            .style(iced::theme::Button::Text)
                                            .on_press(Message::ChoseCharacter(with)),
                                            text("：").size(40)
                                        ],
                                        row![
                                            widget::Svg::new(widget::svg::Handle::from_memory(
                                                include_bytes!("./runtime/quote-left.svg").to_vec()
                                            ))
                                            .width(Length::Fixed(30.0)),
                                            column![text(
                                                cur_comment
                                                    .get("description")
                                                    .unwrap()
                                                    .as_str()
                                                    .expect(
                                                        "Cannot convert `description` into String"
                                                    )
                                            )
                                            .size(30),
                                            text(format!(
                                                "于 {} ",
                                                cur_comment.get("date").unwrap().as_datetime().unwrap()
                                            ))
                                            .size(30)]
                                            .align_items(Alignment::End)
                                        ]
                                    ]);
                                }
                                if !comments.is_empty() {
                                    content = content.push(column![
                                        text("大家对 ta 的评价").size(50),
                                        row![horizontal_space(Length::Fixed(20.0)), lists,]
                                    ]);
                                }
                            }
                            let mut apply_button = row![widget::Button::new(text("返回").size(30))
                                .padding(15)
                                .style(iced::theme::Button::Secondary)
                                .on_press(Message::UnChoose),]
                            .spacing(20);
                            if !choosing::CHARACTERS_WITH_NO_PHOTOS.contains(&chosen) {
                                apply_button = apply_button.push(
                                    widget::Button::new(text("选好啦").size(30))
                                        .padding(15)
                                        .style(iced::theme::Button::Primary)
                                        .on_press(Message::NextStage),
                                );
                            }
                            container(
                                scrollable(row![
                                    horizontal_space(Length::FillPortion(1)),
                                    column![content.spacing(5), apply_button]
                                        .align_items(Alignment::Center)
                                        .width(Length::FillPortion(18)),
                                    horizontal_space(Length::FillPortion(1)),
                                ])
                                .id(choosing::generate_scrollable_id(chosen)),
                            )
                            .center_x()
                            .center_y()
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                        }
                    },
                    Stage::ShowingPlots(displayer) => {
                        let events = displayer.events.lock().unwrap();
                        let experiences = &events[displayer.on_event].experiences;
                        let cur_img = &experiences[events[displayer.on_event].on_experience];
                        row![
                            imageviewer::Viewer::new(cur_img.handle.clone().unwrap())
                                .id(imageviewer::showingplots_viewer_id(
                                    displayer.on_event,
                                    events[displayer.on_event].on_experience
                                ))
                                .width(Length::FillPortion(4))
                                .height(Length::Fill),
                            column![
                                button_from_svg(include_bytes!("./runtime/gears.svg"))
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::OpenSettings),
                                text(events[displayer.on_event].description.clone()).size(50),
                                text(format!("拍摄于 {}", cur_img.shot)).size(30),
                                row![
                                    if displayer.on_event > 0 {
                                        Element::from(
                                            button_from_svg(include_bytes!(
                                                "./runtime/chevron-left.svg"
                                            ))
                                            .width(Length::Fixed(80.0))
                                            .on_press(Message::PreviousEvent),
                                        )
                                    } else {
                                        Element::from(horizontal_space(Length::Fixed(80.0)))
                                    },
                                    if experiences.len() > 1 {
                                        Element::from(column![
                                            button_from_svg(include_bytes!(
                                                "./runtime/chevron-up.svg"
                                            ))
                                            .width(Length::Fixed(60.0))
                                            .on_press(Message::PreviousPhoto),
                                            button_from_svg(include_bytes!(
                                                "./runtime/chevron-down.svg"
                                            ))
                                            .width(Length::Fixed(60.0))
                                            .on_press(Message::NextPhoto)
                                        ])
                                    } else {
                                        Element::from(horizontal_space(Length::Fixed(60.0)))
                                    },
                                    button_from_svg(include_bytes!("./runtime/chevron-right.svg"))
                                        .width(Length::Fixed(80.0))
                                        .on_press(Message::NextEvent),
                                ]
                                .align_items(Alignment::Center),
                                widget::tooltip(
                                    widget::Button::new(text("打开对应文件").size(30))
                                        .padding(10)
                                        .style(iced::theme::Button::Primary)
                                        .on_press(Message::OpenUrl(None)),
                                    "按 O",
                                    widget::tooltip::Position::Bottom
                                )
                                .gap(8)
                                .style(iced::theme::Container::Box),
                                row![
                                    widget::tooltip(
                                        button_from_svg(include_bytes!(
                                            "./runtime/left-to-line.svg"
                                        ),)
                                        .width(Length::Fixed(80.0))
                                        .on_press(Message::BackStage),
                                        "重新选取角色",
                                        widget::tooltip::Position::Bottom
                                    )
                                    .style(iced::theme::Container::Box),
                                    widget::tooltip(
                                        button_from_svg(include_bytes!(
                                            "./runtime/right-to-line.svg"
                                        ),)
                                        .width(Length::Fixed(80.0))
                                        .on_press(Message::NextStage),
                                        "跳过这段时光",
                                        widget::tooltip::Position::Bottom
                                    )
                                    .style(iced::theme::Container::Box)
                                ]
                            ]
                            .spacing(20)
                            .align_items(Alignment::Center)
                            .width(Length::FillPortion(1)),
                        ]
                        .align_items(Alignment::Center)
                        .height(Length::Fill)
                        .into()
                    }
                    Stage::Graduated(vision) => {
                        let images =
                            &vision.images[graduation::ON_LOCATION.load(Ordering::Relaxed)];
                        let displayer =
                            imageviewer::Viewer::new(images.image[vision.on_image].clone())
                                .id(imageviewer::graduation_viewer_id(
                                    graduation::ON_LOCATION.load(Ordering::Relaxed),
                                    vision.on_image,
                                ))
                                .width(Length::Fill)
                                .height(Length::Fill);
                        if vision.show_panel {
                            let mut current = vec![];
                            let mut offsets = vec![];
                            for pan in &vision.images {
                                let pinpoint = |index| {
                                    Element::from(
                                        crate::button_from_svg(
                                            if index
                                                == graduation::ON_LOCATION.load(Ordering::Relaxed)
                                            {
                                                include_bytes!("./runtime/location-check.svg")
                                            } else {
                                                include_bytes!("./runtime/location-pin.svg")
                                            },
                                        )
                                        .width(Length::Fixed(36.0))
                                        .on_press(Message::ClickedPin(index)),
                                    )
                                };
                                current.push(pinpoint);
                                offsets.push(Offset {
                                    x: pan.pinpoint.0,
                                    y: pan.pinpoint.1,
                                });
                            }
                            let map = container(widget::image(image::Handle::from_memory(
                                include_bytes!("./runtime/map.jpg"),
                            )));
                            let pinpointed_map =
                                crate::pinpoint::Pinpoint::new(map, current, offsets);
                            let mut components = column![
                                widget::tooltip(
                                    crate::button_from_svg(include_bytes!(
                                        "./runtime/backward-step.svg"
                                    ))
                                    .width(Length::Fixed(40.0))
                                    .on_press(Message::BackStage),
                                    "返回",
                                    widget::tooltip::Position::Top,
                                )
                                .style(iced::theme::Container::Box),
                                widget::tooltip(
                                    crate::button_from_svg(include_bytes!(
                                        "./runtime/chevron-up.svg"
                                    ))
                                    .width(Length::Fixed(60.0))
                                    .on_press(Message::TogglePanelShown),
                                    "全屏查看",
                                    widget::tooltip::Position::Top,
                                )
                                .gap(-5.0)
                                .style(iced::theme::Container::Box)
                            ]
                            .align_items(Alignment::Center);
                            if images.image.len() > 1 {
                                components = components.push(widget::pick_list(
                                    images.image_names.clone(),
                                    Some(images.image_names[vision.on_image].clone()),
                                    Message::SelectedImage,
                                ));
                            }
                            column![
                                pinpointed_map,
                                row![
                                    container(components).center_y().height(Length::Fill),
                                    displayer
                                ]
                            ]
                            .into()
                        } else {
                            column![overlay::Component::new(displayer, || {
                                column![
                                    vertical_space(Length::Fixed(40.0)),
                                    widget::tooltip(
                                        crate::button_from_svg(include_bytes!(
                                            "./runtime/chevron-down.svg"
                                        ))
                                        .width(Length::Fixed(60.0))
                                        .on_press(Message::TogglePanelShown),
                                        "收起",
                                        widget::tooltip::Position::Top,
                                    )
                                    .gap(-10.0)
                                    .style(iced::theme::Container::Box),
                                ]
                                .into()
                            })]
                            .into()
                        }
                    }
                };
                if state.configs.shown {
                    configs::settings_over(state.configs.clone(), content)
                } else {
                    content
                }
            }
        }
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        match self {
            Memories::Initialization => iced::Subscription::none(),
            Memories::Loading(_) => iced::subscription::events_with(subscriptions::on_loading),
            Memories::Loaded(state) => match state.stage {
                Stage::EntryEvents(_) => {
                    iced::subscription::events_with(subscriptions::on_entry_state)
                }
                Stage::ChoosingCharacter(_) => {
                    iced::subscription::events_with(subscriptions::on_choosing_character)
                }
                Stage::ShowingPlots(_) => {
                    iced::subscription::events_with(subscriptions::on_showing_plots)
                }
                Stage::Graduated(_) => {
                    iced::subscription::events_with(subscriptions::on_graduation)
                }
            },
        }
    }
    fn scale_factor(&self) -> f64 {
        load_scale_factor()
    }

    fn theme(&self) -> Theme {
        match self {
            Memories::Initialization => Theme::Light,
            Memories::Loading(config) => config.theme.clone(),
            Memories::Loaded(state) => state.configs.theme.clone(),
        }
    }
}

pub fn button_from_svg(position: &'static [u8]) -> widget::Button<'static, Message> {
    widget::Button::new(widget::Svg::new(widget::svg::Handle::from_memory(position)))
        .style(iced::theme::Button::Text)
}

fn show_profiles(item: Option<toml::value::Array>, with_name: &str) -> Element<Message> {
    if let Some(item) = item {
        let mut lists = column![];
        for i in &item {
            lists = lists.push(text(i.as_str().unwrap().to_string()).size(30));
        }
        if !item.is_empty() {
            return Element::from(column![
                text(with_name).size(50),
                row![horizontal_space(Length::Fixed(20.0)), lists,]
            ]);
        }
    }
    Element::from(column![])
}
