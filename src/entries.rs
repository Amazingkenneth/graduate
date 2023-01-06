use crate::{EntryState, Error, Memories, Stage, State};
use iced::widget::image;
use iced::Theme;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
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
        let mut preload: Vec<Vec<image::Handle>> = Vec::new();
        preload.resize(cnt as usize, vec![]);
        let m: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(preload));
        // 循环中创建多个线程
        let mut threads = vec![];
        for i in 0..(cnt as usize) {
            // m具有了clone方法
            let m = m.clone();
            let fetching = fetch_files[i]
                .get("image")
                .expect("Cannot get the `image` array..")
                .as_array()
                .expect("Cannot parse image as an array.")
                .to_owned();
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
                let mut preload = m.lock().unwrap();

                // 修改共享内存
                preload[i] = fillin;
            });
            threads.push(t);
        }

        // 等待所有线程结束
        for t in threads {
            t.await?;
        }
        let fetched = m.lock().unwrap();
        Ok(State {
            stage: Stage::EntryEvents(EntryState {
                preload: fetched.to_vec(),
                ..Default::default()
            }),
            idxtable,
            storage: storage.to_string(),
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

pub async fn get_image(
    storage: &String,
    cli: &reqwest::Client,
    url_prefix: &String,
    path: String,
) -> Result<image::Handle, reqwest::Error> {
    println!("Calling get_image({})", path);
    let img_dir = format!("{}{}", storage, path);
    println!("Calling get_image({})", img_dir);
    let img_path = Path::new(&img_dir);
    if img_path.is_file() {
        return Ok(image::Handle::from_path(&img_dir));
    }
    fs::create_dir_all(img_path.parent().expect("Cannot parse the path.")).unwrap();
    async {
        let url = format!("{}{}", url_prefix, path);
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
        let mut file = std::fs::File::create(&img_dir).expect("Failed to create image file.");
        file.write_all(&bytes)
            .expect("Failed to write the image into file in the project directory.");
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
    }
    .await
}
