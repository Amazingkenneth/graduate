use crate::audio::{AudioStream, Audios};
use crate::configs::Configs;
use crate::{audio, EntryState, Error, Memories, Stage, State};
use iced::widget::image;
use iced::Theme;
use reqwest::Client;
use rodio::Sink;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::path::Path;
use std::pin;
use std::sync::{Arc, Mutex};
use toml::value::{self, Datetime};

//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx() -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "Class1", "Graduate").unwrap();
        fs::create_dir_all(proj_dir.data_dir()).unwrap();
        let idxurl: String = format!("https://amazingkenneth.github.io/graduate/index.toml");
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let storage = proj_dir.data_dir().to_str().unwrap().to_string().to_owned();
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
        let fetch_files = idxtable
            .get("event")
            .expect("Cannot get the `event` array.")
            .as_array()
            .expect("Cannot read as an array.")
            .to_owned();
        let fetch_audios = idxtable
            .get("audio")
            .expect("Cannot get the `audio` table.")
            .as_table()
            .expect("Cannot read as a table.")
            .to_owned();
        let cnt = idxtable
            .get("together_events")
            .expect("Didn't find together_events in the indextable.")
            .as_integer()
            .expect("together_events is not an integer");
        let location = idxtable
            .get("url_prefix")
            .expect("Cannot get url prefix.")
            .as_str()
            .expect("Cannot convert into string.")
            .to_string();
        let mut images: Vec<Vec<image::Handle>> = Vec::new();
        images.resize(cnt as usize, vec![]);
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
        for i in 0..(cnt as usize) {
            // m具有了clone方法
            let fetching = fetch_files[i]
                .get("image")
                .expect("Cannot get the `image` array..")
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
                    fillin.push(image::Handle::from_memory(bytes.as_ref().to_vec()));
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
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = ManuallyDrop::new(audio::AudioStream {
            sink: Sink::try_new(&stream_handle).unwrap(),
            stream,
        });
        let sink_mutex = Arc::new(Mutex::new(sink));
        let given_mutex = sink_mutex.clone();
        tokio::spawn(async move {
            audio::play_music(given_mutex, audio_paths).await;
        });
        let fetched = img_mutex.lock().unwrap();
        // let try_mutex = sink_mutex.clone();
        Ok(State {
            stage: Stage::EntryEvents(EntryState {
                preload: fetched.to_vec(),
                ..Default::default()
            }),
            idxtable,
            storage: storage.to_string(),
            configs: Configs {
                scale_factor: 1.0,
                theme: Theme::Light,
                from_date: Datetime {
                    date: Some(toml::value::Date {
                        year: 2020,
                        month: 6,
                        day: 1,
                    }),
                    time: None,
                    offset: None,
                },
                aud_module: sink_mutex,
                show: false,
            },
        })
    }
    pub fn get_current_event(&self, on_event: usize) -> toml::value::Value {
        self.idxtable
            .get("event")
            .expect("Cannot get the `event` array.")
            .as_array()
            .expect("Cannot read as an array.")[on_event]
            .to_owned()
    }
}
