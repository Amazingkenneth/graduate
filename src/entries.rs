use crate::audio::{self, AudioStream};
use crate::configs::{self, Configs};
use crate::visiting::ShootingTime;
use crate::{choosing, EntryState, Error, Memories, Stage, State};
use iced::widget::image;
use iced::Theme;
use reqwest::Client;
use rodio::Sink;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx() -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "Class1", "Graduate").unwrap();
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let config_path = format!("{}{}", proj_dir.config_dir().display(), "/configs.toml");
        let storage: String = proj_dir.data_dir().display().to_string();
        println!("storage: {}", storage);
        fs::create_dir_all(&storage).unwrap();
        fs::create_dir_all(proj_dir.config_dir().display().to_string()).unwrap();
        let idxurl: String = format!("https://graduate-cdn.netlify.com/index.toml");
        let cli = Client::new().to_owned();
        let content = cli
            .get(&idxurl)
            .send()
            .await
            .expect("Cannot send request")
            .text()
            .await
            .expect("Cannot convert response to text.");
        let mut buffer = File::create(idxdir).unwrap();
        buffer
            .write_all(content.as_bytes())
            .expect("Cannot write into file.");
        let idxtable = content
            .parse::<toml::value::Value>()
            .expect("Cannot parse the content.")
            .as_table()
            .expect("Cannot read as table.")
            .to_owned();
        let together_events = idxtable
            .get("together_event")
            .expect("Cannot get the `together_event` array.")
            .as_array()
            .expect("Cannot read as an array.")
            .to_owned();
        let fetch_audios = idxtable
            .get("audio")
            .expect("Cannot get the `audio` table.")
            .as_table()
            .expect("Cannot read as a table.")
            .to_owned();
        let location = idxtable
            .get("url_prefix")
            .expect("Cannot get url prefix.")
            .as_str()
            .expect("Cannot convert into string.")
            .to_string();
        let mut images: Vec<Vec<image::Handle>> = Vec::new();
        images.resize(together_events.len(), vec![]);
        let img_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(images));
        let audios: Vec<String> = Vec::new();
        let aud_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(audios));
        // 循环中创建多个线程
        let mut threads = vec![];
        for audio_type in fetch_audios.values() {
            let cur_audios = audio_type
                .as_array()
                .expect("cannot parse into an array")
                .to_owned();
            for fetching in cur_audios {
                let aud_mutex = aud_mutex.clone();
                let location = location.clone();
                let relative_path = fetching
                    .as_str()
                    .expect("Cannot read as a string")
                    .to_owned()
                    .to_string();
                let audio_dir = format!("{}{}", &storage, relative_path);
                let t = tokio::spawn(async move {
                    let cli = Client::new();
                    let audio_path = Path::new(&audio_dir);
                    if !audio_path.is_file() {
                        fs::create_dir_all(audio_path.parent().expect("Cannot parse the path."))
                            .unwrap();
                        let url = format!("{}{}", location, relative_path);
                        // println!("url: {}", url);
                        let bytes = cli
                            .get(&url)
                            .send()
                            .await
                            .expect("Cannot send request")
                            .bytes()
                            .await
                            .expect("Cannot read the image into bytes.");
                        println!("Done processing image!");
                        let mut file = std::fs::File::create(&audio_dir)
                            .expect("Failed to create image file.");
                        file.write_all(&bytes).expect(
                            "Failed to write the audio into file in the project directory.",
                        );
                    }
                    let mut aud_paths = aud_mutex.lock().unwrap();
                    aud_paths.push(audio_dir);
                });
                threads.push(t);
            }
        }
        for (i, cur_image) in together_events.iter().enumerate() {
            let fetching = cur_image
                .get("image")
                .expect("Cannot get the `image` array.")
                .as_array()
                .expect("Cannot parse image as an array.")
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
                    fs::create_dir_all(img_path.parent().expect("Cannot parse the path.")).unwrap();
                    let url = format!("{}{}", location, relative_path);
                    println!("url: {}", url);
                    let bytes = cli
                        .get(&url)
                        .send()
                        .await
                        .expect("Cannot send request")
                        .bytes()
                        .await
                        .expect("Cannot read the image into bytes.");
                    println!("Done processing image!");
                    let mut file =
                        std::fs::File::create(&img_dir).expect("Failed to create image file.");
                    file.write_all(&bytes)
                        .expect("Failed to write the image into file in the project directory.");
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

        let audio_paths: Vec<String> = std::mem::take(&mut aud_mutex.lock().unwrap());
        let given_paths = audio_paths.clone();
        let sink = {
            let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
            ManuallyDrop::new(audio::AudioStream {
                sink: Sink::try_new(&stream_handle).unwrap(),
                stream,
            })
        };
        let sink_mutex = Arc::new(Mutex::new(sink));
        let given_mutex = sink_mutex.clone();
        let daemon_status = Arc::new(AtomicBool::new(true));
        let given_status = daemon_status.clone();

        let fetched = Arc::new(Mutex::new(img_mutex.lock().unwrap().to_vec()));
        if let Ok(init_configs) = std::fs::read_to_string(&config_path) {
            let config_table = init_configs
                .parse::<toml::value::Value>()
                .unwrap()
                .as_table()
                .expect("Cannot read as table.")
                .to_owned();
            let initial_volume = config_table
                .get("audio-volume")
                .unwrap()
                .as_float()
                .unwrap() as f32;
            if config_table
                .get("audio-playing")
                .unwrap()
                .as_bool()
                .unwrap()
            {
                tokio::spawn(async move {
                    audio::play_music(given_mutex, given_paths, given_status, initial_volume).await;
                });
            }
            let theme = if config_table.get("light-theme").unwrap().as_bool().unwrap() {
                Theme::Light
            } else {
                Theme::Dark
            };
            let scale_factor = config_table
                .get("scale-factor")
                .unwrap()
                .as_float()
                .unwrap();
            let from_date: ShootingTime = config_table
                .get("from-date")
                .unwrap()
                .as_datetime()
                .unwrap()
                .into();
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
                        State {
                            stage: Stage::EntryEvents(EntryState {
                                preload: fetched,
                                ..Default::default()
                            }),
                            idxtable,
                            storage,
                            configs: Configs {
                                scale_factor,
                                theme,
                                from_date,
                                aud_volume: 1.0,
                                aud_module: sink_mutex,
                                daemon_running: daemon_status,
                                audio_paths,
                                config_path,
                                shown: false,
                            },
                        },
                    )
                    .await
                    .unwrap();
                    return Ok(res);
                }
                _ => Stage::EntryEvents(EntryState {
                    // or "EntryEvents"
                    preload: fetched,
                    ..Default::default()
                }),
            };

            Ok(State {
                stage,
                idxtable,
                storage,
                configs: Configs {
                    scale_factor,
                    theme,
                    from_date,
                    aud_volume: 1.0,
                    aud_module: sink_mutex,
                    daemon_running: daemon_status,
                    audio_paths,
                    config_path,
                    shown: false,
                },
            })
        } else {
            tokio::spawn(async move {
                audio::play_music(given_mutex, given_paths, given_status, 1.0).await;
            });
            Ok(State {
                stage: Stage::EntryEvents(EntryState {
                    preload: fetched,
                    ..Default::default()
                }),
                idxtable,
                storage,
                configs: Configs {
                    scale_factor: 1.0,
                    theme: Theme::Light,
                    from_date: crate::visiting::ShootingTime::Precise(
                        time::macros::datetime!(2020-06-01 0:00),
                    ),
                    aud_volume: 1.0,
                    aud_module: sink_mutex,
                    daemon_running: daemon_status,
                    audio_paths,
                    config_path,
                    shown: false,
                },
            })
        }
    }
    pub fn get_current_event(&self, on_event: usize) -> toml::value::Value {
        self.idxtable
            .get("together_event")
            .expect("Cannot get the `event` array.")
            .as_array()
            .expect("Cannot read as an array.")[on_event]
            .to_owned()
    }
}
