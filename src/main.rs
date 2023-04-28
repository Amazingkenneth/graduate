#![allow(dead_code, unused_imports)]
mod audio;
mod choosing;
mod configs;
mod entries;
mod graduation;
mod subscriptions;
mod visiting;

use ::image::ImageFormat;
use configs::Configs;
use iced::widget::{
    self, column, container, horizontal_space, image, row, scrollable, text, text_input,
    vertical_rule, vertical_space, Column, Row,
};
use iced::window::Mode;
use iced::{
    alignment, subscription, window, Alignment, Application, Color, Command, Element, Event,
    Length, Settings, Theme,
};
use rand::Rng;
use reqwest::Client;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use time::PrimitiveDateTime;
use toml::value::Table;

pub static DELETE_FILES_ON_EXIT: AtomicBool = AtomicBool::new(false);

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
            size: (1400, 800),
            icon: Some(
                iced::window::icon::from_file_data(
                    include_bytes!("./runtime/icon.png"),
                    Some(ImageFormat::Png),
                )
                .unwrap(),
            ),
            ..window::Settings::default()
        },
        default_font: Some(include_bytes!("./YEFONTFuJiYaTi-3.ttf")),
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
    description: String,
    previous_stage: EntryState,
    element_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct VisitingState {
    character_name: String,
    on_event: usize,
    events: Arc<Mutex<Vec<visiting::Event>>>,
    // images: VecDeque<visiting::ImgLoader>,
}

#[derive(Clone, Debug, Default)]
pub struct GraduationState {
    on: i32,
}

#[derive(Clone, Debug)]
pub enum Message {
    BackStage,
    ChoseCharacter(usize),
    DescriptionEdited(String),
    EscapeFullScreen,
    FetchImage(Result<Memories, Error>),
    FinishedTyping,
    HideSettings,
    IsDarkTheme(bool),
    Loaded(Result<State, Error>),
    LoadedImage(Result<EntryState, Error>),
    ModifyVolume(f32),
    NextEvent,
    NextPerson,
    NextPhoto,
    NextSong,
    NextStage,
    OpenInExplorer,
    OpenSettings,
    PreviousEvent,
    PreviousPerson,
    PreviousPhoto,
    Refresh,
    ScaleDown,
    ScaleEnlarge,
    ScaleRestore,
    SwitchDeleteFilesStatus,
    SwitchMusicStatus,
    ToggleMode,
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
            Command::perform(State::get_idx(), Message::Loaded),
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
                Stage::Graduated(_) => "就这样，我们毕业啦",
            },
            _ => "加载中",
        };
        format!("{} - 有你，才是一班。", subtitle)
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        println!("On update()");
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
                        config.scale_factor /= 1.05;
                    }
                    Message::ScaleEnlarge => {
                        config.scale_factor *= 1.05;
                    }
                    Message::ScaleRestore => {
                        config.scale_factor = 1.0;
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
                        if config
                            .daemon_running
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let sink = &config.aud_module.lock().unwrap().sink;
                            if sink.is_paused() {
                                sink.play();
                            } else {
                                sink.pause();
                            }
                        } else {
                            let (stream, stream_handle) =
                                rodio::OutputStream::try_default().unwrap();
                            let audio_stream = std::mem::ManuallyDrop::new(audio::AudioStream {
                                sink: rodio::Sink::try_new(&stream_handle).unwrap(),
                                stream,
                            });
                            config.aud_module = Arc::new(Mutex::new(audio_stream));
                            let given_mutex = config.aud_module.clone();
                            config.daemon_running.store(true, Ordering::Relaxed);
                            let running_status = config.daemon_running.clone();
                            let paths = config.audio_paths.clone();
                            tokio::spawn(async {
                                audio::play_music(given_mutex, paths, running_status, 1.0).await;
                            });
                        }
                        return Command::none();
                    }
                    Message::ModifyVolume(new_volume) => {
                        config.volume_percentage = new_volume;
                        if config
                            .daemon_running
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let sink = &config.aud_module.lock().unwrap().sink;
                            sink.set_volume(new_volume / 100.0);
                        }
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
                        state.configs.scale_factor /= 1.05;
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::ScaleEnlarge => {
                        state.configs.scale_factor *= 1.05;
                        configs::save_configs(state);
                        return Command::none();
                    }
                    Message::ScaleRestore => {
                        state.configs.scale_factor = 1.0;
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
                        if state
                            .configs
                            .daemon_running
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let sink = &state.configs.aud_module.lock().unwrap().sink;
                            if sink.is_paused() {
                                sink.play();
                            } else {
                                sink.pause();
                            }
                        } else {
                            let (stream, stream_handle) =
                                rodio::OutputStream::try_default().unwrap();
                            let audio_stream = std::mem::ManuallyDrop::new(audio::AudioStream {
                                sink: rodio::Sink::try_new(&stream_handle).unwrap(),
                                stream,
                            });
                            state.configs.aud_module = Arc::new(Mutex::new(audio_stream));
                            let given_mutex = state.configs.aud_module.clone();
                            state.configs.daemon_running.store(true, Ordering::Relaxed);
                            let running_status = state.configs.daemon_running.clone();
                            let paths = state.configs.audio_paths.clone();
                            tokio::spawn(async {
                                audio::play_music(given_mutex, paths, running_status, 1.0).await;
                            });
                        }
                        return Command::none();
                    }
                    Message::ModifyVolume(new_volume) => {
                        state.configs.volume_percentage = new_volume;
                        if state
                            .configs
                            .daemon_running
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            let sink = &state.configs.aud_module.lock().unwrap().sink;
                            sink.set_volume(new_volume / 100.0);
                        }
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
                            Stage::ShowingPlots(displayer) => {
                                let events = displayer.events.lock().unwrap();
                                let on_image = &events[displayer.on_event].on_experience;
                                format!(
                                    "{}{}",
                                    state.storage,
                                    events[displayer.on_event].experiences[*on_image].path
                                )
                            }
                            _ => String::from(""),
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
                    Message::BackStage | Message::NextStage => {
                        configs::save_configs(state);
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
                                    .expect("No date value in the item.")
                                    .as_datetime()
                                    .unwrap()
                                    .into();
                                let state = state.clone();
                                *self = Memories::Loading(state.configs.clone());
                                return Command::perform(
                                    choosing::get_configs(None, state),
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
                                        choosing::generate_id(chosen),
                                        scrollable::RelativeOffset::START,
                                    );
                                }
                                _ => {}
                            },
                        }
                        Command::none()
                    }
                    Stage::ShowingPlots(ref mut displayer) => {
                        match message {
                            Message::PreviousEvent => {
                                displayer.on_event -= 1;
                                let need_force_run = {
                                    displayer.events.lock().unwrap()[displayer.on_event]
                                        .get_image_handle()
                                };
                                if let None = need_force_run {
                                    let handle = {
                                        let events = displayer.events.lock().unwrap();
                                        events[displayer.on_event].get_join_handle()
                                    };
                                    let memo = std::mem::replace(self, Memories::Initialization);
                                    return Command::perform(
                                        visiting::force_load(handle, memo),
                                        Message::FetchImage,
                                    );
                                }
                                visiting::load_images(state);
                            }
                            Message::NextEvent => {
                                displayer.on_event += 1;
                                if displayer.on_event == displayer.events.lock().unwrap().len() {
                                    let state = state.to_owned();
                                    *self = Memories::Loading(state.configs.clone());
                                    return Command::perform(
                                        graduation::load_map(state),
                                        Message::Loaded,
                                    );
                                }
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
                            Message::PreviousPhoto => {
                                let mut events = displayer.events.lock().unwrap();
                                let len = &events[displayer.on_event].experiences.len();
                                let on_image = &mut events[displayer.on_event].on_experience;
                                *on_image = (*on_image + len - 1) % len;
                            }
                            Message::NextPhoto => {
                                let mut events = displayer.events.lock().unwrap();
                                let len = &events[displayer.on_event].experiences.len();
                                let on_image = &mut events[displayer.on_event].on_experience;
                                *on_image = (*on_image + 1) % len;
                            }
                            Message::NextStage => {
                                let state = state.clone();
                                *self = Memories::Loading(state.configs.clone());
                                return Command::perform(
                                    graduation::load_map(state),
                                    Message::Loaded,
                                );
                            }
                            Message::BackStage => {
                                return Command::perform(State::get_idx(), Message::Loaded);
                            }
                            _ => {}
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
            Memories::Loading(_) | Memories::Initialization => {
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
                let content = match &state.stage {
                    Stage::EntryEvents(chosen) => row![
                        image::viewer(
                            chosen.preload.lock().unwrap()[chosen.on_event as usize]
                                [chosen.on_image]
                                .clone()
                        )
                        .width(Length::Fixed(1400.0)),
                        column![
                            button_from_svg(include_bytes!("./runtime/gears.svg").to_vec())
                                .width(Length::Fixed(80.0))
                                .on_press(Message::OpenSettings),
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
                                .width(Length::Fixed(80.0))
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
                                        .width(Length::Fixed(40.0))
                                        .on_press(Message::PreviousPhoto),
                                        button_from_svg(
                                            include_bytes!("./runtime/down.svg").to_vec()
                                        )
                                        .width(Length::Fixed(40.0))
                                        .on_press(Message::NextPhoto)
                                    ])
                                } else {
                                    Element::from(horizontal_space(Length::Fixed(40.0)))
                                },
                                button_from_svg(
                                    include_bytes!("./runtime/arrow-right.svg").to_vec()
                                )
                                .width(Length::Fixed(80.0))
                                .on_press(Message::NextEvent),
                            ],
                            widget::tooltip(
                                widget::Button::new(text("打开对应文件").size(30))
                                    .padding(10)
                                    .style(iced::theme::Button::Secondary)
                                    .on_press(Message::OpenInExplorer),
                                "按 O",
                                widget::tooltip::Position::FollowCursor
                            )
                            .style(iced::theme::Container::Box),
                        ]
                        .spacing(20)
                        .align_items(Alignment::Center)
                        .width(Length::Fill),
                    ]
                    .align_items(Alignment::Center)
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
                                heads.last_mut().expect("Cannot get the last value.").push(
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
                            let content = scrollable(column![searchbox, scroll_head,].spacing(10));
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
                            content = content.push(show_profiles(profile.nickname, "ta 的昵称"));
                            if let Some(relations) = profile.relationship {
                                let mut lists = column![];
                                for relation in &relations {
                                    let cur_relation =
                                        relation.as_table().expect("Cannot read as a table");
                                    let author = cur_relation
                                        .get("by")
                                        .expect("Cannot get `with`")
                                        .as_integer()
                                        .expect("`with` isn't a valid integer")
                                        as usize;
                                    lists = lists.push(row![
                                        text(format!("是 ",)).size(30),
                                        widget::Button::new(
                                            text(choosing.avatars[author].name.clone()).size(30)
                                        )
                                        .padding(0)
                                        .on_press(Message::ChoseCharacter(author))
                                        .style(iced::theme::Button::Text),
                                        text(format!(
                                            " 的 {}；",
                                            cur_relation
                                                .get("is")
                                                .expect("Cannot get `is`")
                                                .as_str()
                                                .unwrap()
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
                            for i in &choosing.avatars[chosen].emoji {
                                emojis = emojis.push(
                                    column![
                                        widget::image::viewer(i.emoji.clone())
                                            .height(Length::Fixed(400.0)),
                                        text(i.emoji_name.clone()).size(30)
                                    ]
                                    .align_items(Alignment::Center),
                                );
                            }
                            let mut content = column![row![
                                content,
                                scrollable(emojis)
                                    .horizontal_scroll(iced::widget::scrollable::Properties::new()) // .height(Length::Shrink)
                            ]];
                            content = content.push(show_profiles(profile.plots, "ta 的小日常"));
                            if let Some(intro) = profile.introduction {
                                content = content.push(column![
                                    text("ta 的自传").size(50),
                                    row![
                                        horizontal_space(Length::Fixed(20.0)),
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
                                                text(choosing.avatars[with].name.clone()).size(40)
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
                                                    .expect("Cannot get `description`")
                                                    .as_str()
                                                    .expect(
                                                        "Cannot convert `description` into String"
                                                    )
                                            )
                                            .size(30),
                                            text(format!(
                                                "于 {} ",
                                                cur_comment.get("date").expect("Cannot get `date`").as_datetime().unwrap()
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
                            let apply_button = row![
                                widget::Button::new(text("返回").size(30))
                                    .padding(15)
                                    .style(iced::theme::Button::Secondary)
                                    .on_press(Message::UnChoose),
                                widget::Button::new(text("选好啦").size(30))
                                    .padding(15)
                                    .style(iced::theme::Button::Primary)
                                    .on_press(Message::NextStage),
                            ]
                            .spacing(20);
                            container(
                                scrollable(
                                    column![content.spacing(5), apply_button]
                                        .align_items(Alignment::Center)
                                        .width(Length::Fill),
                                )
                                .id(choosing::generate_id(chosen)),
                            )
                            .center_x()
                            .center_y()
                            .into()
                        }
                    },
                    Stage::ShowingPlots(displayer) => {
                        let events = displayer.events.lock().unwrap();
                        let experiences = &events[displayer.on_event].experiences;
                        let cur_img = &experiences[events[displayer.on_event].on_experience];
                        let main_image = image::viewer(cur_img.handle.clone().unwrap())
                            .width(Length::Fixed(1400.0));
                        row![
                            main_image,
                            column![
                                button_from_svg(include_bytes!("./runtime/gears.svg").to_vec())
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::OpenSettings),
                                text(events[displayer.on_event].description.clone()).size(50),
                                text(format!("拍摄于 {}", cur_img.shot)).size(30),
                                row![
                                    if displayer.on_event > 0 {
                                        Element::from(
                                            button_from_svg(
                                                include_bytes!("./runtime/chevron-left.svg")
                                                    .to_vec(),
                                            )
                                            .width(Length::Fixed(80.0))
                                            .on_press(Message::PreviousEvent),
                                        )
                                    } else {
                                        Element::from(horizontal_space(Length::Fixed(80.0)))
                                    },
                                    if experiences.len() > 1 {
                                        Element::from(column![
                                            button_from_svg(
                                                include_bytes!("./runtime/chevron-up.svg").to_vec()
                                            )
                                            .width(Length::Fixed(60.0))
                                            .on_press(Message::PreviousPhoto),
                                            button_from_svg(
                                                include_bytes!("./runtime/chevron-down.svg")
                                                    .to_vec()
                                            )
                                            .width(Length::Fixed(60.0))
                                            .on_press(Message::NextPhoto)
                                        ])
                                    } else {
                                        Element::from(horizontal_space(Length::Fixed(60.0)))
                                    },
                                    button_from_svg(
                                        include_bytes!("./runtime/chevron-right.svg").to_vec()
                                    )
                                    .width(Length::Fixed(80.0))
                                    .on_press(Message::NextEvent),
                                ]
                                .align_items(Alignment::Center),
                                widget::tooltip(
                                    widget::Button::new(text("打开对应文件").size(30))
                                        .padding(10)
                                        .style(iced::theme::Button::Secondary)
                                        .on_press(Message::OpenInExplorer),
                                    "按 O",
                                    widget::tooltip::Position::FollowCursor
                                )
                                .style(iced::theme::Container::Box),
                                widget::Button::new(
                                    column![text("回到最初").size(28), text("那样……").size(28)]
                                        .spacing(10)
                                )
                                .style(iced::theme::Button::Positive)
                                .on_press(Message::BackStage)
                                .padding(10),
                                widget::Button::new(text("跳过这段时光").size(28))
                                    .style(iced::theme::Button::Primary)
                                    .on_press(Message::NextStage)
                                    .padding(10)
                            ]
                            .spacing(20)
                            .align_items(Alignment::Center)
                            .width(Length::Fill),
                        ]
                        .align_items(Alignment::Center)
                        .into()
                    }
                    _ => {
                        println!("current state: {:?}", state);
                        row![column![text("Not implemented").size(50)].spacing(20)].into()
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
        // from iced_native::events_with
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
                    iced::subscription::events_with(subscriptions::on_graduation)
                }
                _ => iced::Subscription::none(),
            },
        }
    }
    fn scale_factor(&self) -> f64 {
        match self {
            Memories::Initialization => 1.0,
            Memories::Loading(config) => config.scale_factor,
            Memories::Loaded(state) => state.configs.scale_factor,
        }
    }

    fn theme(&self) -> Theme {
        match self {
            Memories::Initialization => Theme::Light,
            Memories::Loading(config) => config.theme.clone(),
            Memories::Loaded(state) => state.configs.theme.clone(),
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
                row![horizontal_space(Length::Fixed(20.0)), lists,]
            ]);
        }
    }
    Element::from(column![])
}
