use crate::audio;
use crate::configs::Configs;
use crate::visiting::ShootingTime;
use crate::{choosing, EntryState, Stage, State};
use iced::widget::image;
use iced::Theme;
use reqwest::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx(reusable: Option<State>) -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "Class1", "Graduate").unwrap();
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let config_path = format!("{}{}", proj_dir.config_dir().display(), "/configs.toml");
        let storage: String = proj_dir.data_dir().display().to_string();
        dbg!(&storage);
        let idxurl = String::from("https://yankang1.coding.net/p/graduate/shared-depot/graduate/git/raw/gh-pages/index.toml");
        let content = if let None = reusable {
            fs::create_dir_all(&storage).unwrap();
            fs::create_dir_all(proj_dir.config_dir().display().to_string()).unwrap();
            let cli = Client::new().to_owned();
            if let Ok(fetching) = cli.get(&idxurl).send().await {
                let content = fetching.text().await.unwrap();
                let mut buffer = File::create(idxdir).unwrap();
                buffer.write_all(content.as_bytes()).unwrap();
                content
            } else {
                fs::read_to_string(idxdir).unwrap()
            }
        } else {
            fs::read_to_string(idxdir).unwrap()
        };
        let idxtable = content
            .parse::<toml::value::Value>()
            .unwrap()
            .as_table()
            .unwrap()
            .to_owned();
        let together_events = idxtable
            .get("together_event")
            .unwrap()
            .as_array()
            .unwrap()
            .to_owned();
        let location = idxtable
            .get("url_prefix")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let mut images: Vec<Vec<image::Handle>> = Vec::new();
        images.resize(together_events.len(), vec![]);
        let img_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(images));
        let audios: Vec<String> = Vec::new();
        let aud_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(audios));

        let mut threads = vec![];
        if let None = reusable {
            let fetch_audios = idxtable
                .get("audio")
                .unwrap()
                .as_table()
                .unwrap()
                .to_owned();
            // 循环中创建多个线程
            for audio_type in fetch_audios.values() {
                let cur_audios = audio_type.as_array().unwrap().to_owned();
                for fetching in cur_audios {
                    let aud_mutex = aud_mutex.clone();
                    let location = location.clone();
                    let relative_path = fetching.as_str().unwrap().to_owned().to_string();
                    let audio_dir = format!("{}{}", &storage, relative_path);
                    let t = tokio::spawn(async move {
                        let cli = Client::new();
                        let audio_path = Path::new(&audio_dir);
                        if !audio_path.is_file() {
                            fs::create_dir_all(audio_path.parent().unwrap()).unwrap();
                            let url = format!("{}{}", location, relative_path);
                            let bytes = cli.get(&url).send().await.unwrap().bytes().await.unwrap();
                            let mut file = std::fs::File::create(&audio_dir).unwrap();
                            file.write_all(&bytes).unwrap();
                        }
                        let mut aud_paths = aud_mutex.lock().unwrap();
                        aud_paths.push(audio_dir);
                    });
                    threads.push(t);
                }
            }
        }
        for (i, cur_image) in together_events.iter().enumerate() {
            let fetching = cur_image
                .get("image")
                .unwrap()
                .as_array()
                .unwrap()
                .to_owned();
            let img_mutex = img_mutex.clone();
            let storage = storage.clone();
            let location = location.clone();
            // 创建线程
            let t = tokio::spawn(async move {
                // Arc类型可以直接使用内部的值，从信号量中取得共享内存的方法与不使用Arc完全一致
                let cli = Client::new();
                let mut fillin: Vec<image::Handle> = Vec::new();
                for cur_image in fetching {
                    let mut img_path_string = cur_image.to_owned().to_string();
                    img_path_string.pop();
                    let relative_path = img_path_string.split_off(1);
                    let img_dir = format!("{}{}", &storage, relative_path);
                    let img_path = Path::new(&img_dir);
                    if img_path.is_file() {
                        fillin.push(image::Handle::from_path(&img_dir));
                        continue;
                    }
                    fs::create_dir_all(img_path.parent().unwrap()).unwrap();
                    let url = format!("{}{}", location, relative_path);
                    let bytes = cli.get(&url).send().await.unwrap().bytes().await.expect(
                        format!("Panics when trying to fetch image of path: {img_dir}").as_str(),
                    );
                    let mut file = std::fs::File::create(&img_dir).unwrap();
                    file.write_all(&bytes).unwrap();
                    fillin.push(image::Handle::from_memory(bytes));
                }
                let mut images = img_mutex.lock().unwrap();

                // 修改共享内存
                images[i] = fillin;
            });
            threads.push(t);
        }
        // 等待所有线程结束
        for t in threads {
            t.await?;
        }
        let fetched = Arc::new(Mutex::new(img_mutex.lock().unwrap().to_vec()));
        if let Some(state) = reusable {
            return Ok(State {
                stage: Stage::EntryEvents(EntryState {
                    preload: fetched,
                    ..Default::default()
                }),
                ..state
            });
        }
        let audio_paths: Vec<String> = std::mem::take(&mut aud_mutex.lock().unwrap());
        let _ = {
            let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            *audio::AUDIO_PLAYER.lock().unwrap() = Some(audio::AudioStream {
                sink: crate::sink::Sink::try_new(&stream_handle).unwrap(),
                stream,
                audio_paths,
            });
        };

        if let Ok(init_configs) = std::fs::read_to_string(&config_path) {
            let config_table = init_configs
                .parse::<toml::value::Value>()
                .unwrap()
                .as_table()
                .unwrap()
                .to_owned();
            let initial_volume = config_table
                .get("volume-percentage")
                .unwrap()
                .as_float()
                .unwrap() as f32;
            audio::AUDIO_PLAYER
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .sink
                .set_volume(initial_volume / 100.0);
            if config_table.get("audio-paused").unwrap().as_bool().unwrap() {
                audio::AUDIO_PLAYER
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .sink
                    .pause();
            }
            std::thread::spawn(|| {
                audio::play_music();
            });
            let theme = if config_table.get("light-theme").unwrap().as_bool().unwrap() {
                Theme::Light
            } else {
                Theme::Dark
            };
            crate::store_scale_factor(
                config_table
                    .get("scale-factor")
                    .unwrap()
                    .as_float()
                    .unwrap(),
            );
            let from_date: ShootingTime = config_table
                .get("from-date")
                .unwrap()
                .as_datetime()
                .unwrap()
                .into();
            let configs = Configs {
                theme,
                from_date,
                volume_percentage: initial_volume,
                config_path,
                shown: false,
                full_screened: false,
                id: iced::window::Id::unique(),
            };
            let stage = match config_table.get("stage").unwrap().as_str().unwrap() {
                "ChoosingCharacter" => {
                    let on_character = {
                        let on = config_table
                            .get("on_character")
                            .unwrap()
                            .as_integer()
                            .unwrap();
                        if on == -1 {
                            None
                        } else {
                            Some(on as usize)
                        }
                    };
                    let res = choosing::get_configs(
                        on_character,
                        iced::widget::scrollable::RelativeOffset::START,
                        State {
                            stage: Stage::EntryEvents(EntryState {
                                preload: fetched,
                                ..Default::default()
                            }),
                            idxtable,
                            storage,
                            configs,
                        },
                    )
                    .await
                    .unwrap();
                    return Ok(res);
                }
                "Graduated" => {
                    let res = crate::graduation::load_map(State {
                        stage: Stage::Graduated(crate::GraduationState {
                            ..Default::default()
                        }),
                        idxtable,
                        storage,
                        configs,
                    })
                    .await
                    .unwrap();
                    return Ok(res);
                }
                _ => Stage::EntryEvents(EntryState {
                    preload: fetched,
                    ..Default::default()
                }),
            };
            Ok(State {
                stage,
                idxtable,
                storage,
                configs,
            })
        } else {
            std::thread::spawn(|| {
                audio::play_music();
            });
            Ok(State {
                stage: Stage::EntryEvents(EntryState {
                    preload: fetched,
                    ..Default::default()
                }),
                idxtable,
                storage,
                configs: Configs {
                    theme: Theme::Light,
                    from_date: crate::visiting::ShootingTime::Precise(
                        time::macros::datetime!(2020-06-01 0:00),
                    ),
                    volume_percentage: 100.0,
                    config_path,
                    shown: false,
                    full_screened: false,
                    id: iced::window::Id::unique(),
                },
            })
        }
    }
    pub fn get_current_event(&self, on_event: usize) -> toml::value::Value {
        self.idxtable
            .get("together_event")
            .unwrap()
            .as_array()
            .unwrap()[on_event]
            .to_owned()
    }
}
