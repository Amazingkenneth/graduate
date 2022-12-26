use crate::{EntryState, Error, Memories, Stage, State};
use iced::widget::image;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use toml::value;

//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx() -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "Class1", "Graduate").unwrap();
        fs::create_dir_all(proj_dir.data_dir()).unwrap();
        let location = "https://graduate-1313398930.cos.ap-guangzhou.myqcloud.com";
        let idxurl: String = format!("https://amazingkenneth.github.io/graduate/index.toml");
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let storage = proj_dir.data_dir().to_str().unwrap().to_string();
        let cli = Client::new();
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
            .expect("Cannot read as an array.");
        let cnt = idxtable
            .get("together_events")
            .expect("Didn't find together_events in the indextable.")
            .as_integer()
            .expect("together_events is not an integer");
        let mut preload: Vec<Vec<image::Handle>> = Vec::with_capacity(cnt as usize);
        //preload.resize(cnt as usize);
        for i in 0..(cnt as usize) {
            let mut img_path_string = fetch_files[i]
                .get("image")
                .expect("Cannot get the `image` array..")
                .as_array()
                .expect("Cannot parse image as an array.")[0]
                .to_owned()
                .to_string();
            img_path_string.pop();
            let relative_path = img_path_string.split_off(1);
            let img_dir = format!("{}{}", storage, relative_path);
            let img_path = Path::new(&img_dir);
            if img_path.is_file() {
                preload.push(vec![image::Handle::from_path(&img_dir)]);
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
            let mut file = std::fs::File::create(&img_dir).expect("Failed to create image file.");
            file.write_all(&bytes)
                .expect("Failed to write the image into file in the project directory.");
            preload.push(vec![image::Handle::from_memory(bytes.as_ref().to_vec())]);
        }
        Ok(State {
            stage: Stage::EntryEvents(EntryState {
                preload,
                ..Default::default()
            }),
            idxtable,
            storage,
            client: cli,
            url_prefix: location.to_string(),
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

pub async fn get_photos(mut state: State) -> Result<State, crate::Error> {
    match state.stage {
        Stage::EntryEvents(ref mut chosen) => {
            let img_array = state
                .idxtable
                .get("event")
                .expect("Cannot get the `event` array.")
                .as_array()
                .expect("Cannot read as an array.")[chosen.on_event]
                .get("image")
                .expect("No image value in the item.")
                .as_array()
                .expect("Cannot read the path.")
                .to_owned();
            //let mut preload = chosen.preload[chosen.on_event].to_owned();
            for photo in 1..(img_array.len() as usize) {
                let handle = get_image(
                    &state.storage,
                    &state.client,
                    &state.url_prefix,
                    img_array[photo].to_string(),
                )
                .await
                .expect("Cannot get image.");
                    chosen.preload[chosen.on_event].push(handle);
            }
            //chosen.preload[chosen.on_event]
            Ok(state)
        }
        _ => Ok(state),
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

/*
pub async fn change_image(
    state: State,
    mut to_event: i64,
    to_image: i64,
) -> Result<State, crate::Error> {
    match state.stage {
        Stage::EntryEvents(ref chosen) => {
            let cnt = chosen.preload.len() as i64;
            let mut chosen = chosen.clone();
            let img_array = state
                .get_current_event(chosen.on_event)
                .get("image")
                .expect("No image value in the item.")
                .as_array()
                .expect("Cannot read the path.")
                .to_owned();
            to_event = (to_event + cnt) % cnt;
            chosen.on_event = to_event as u32;
            chosen.on_image = ((to_image as usize) + img_array.len()) % img_array.len();
            let mut img_path = img_array[chosen.on_image].to_owned().to_string();
            img_path.pop();
            chosen.image = state
                .get_image(img_path.split_off(1))
                .await
                .expect("Cannot get image.");
            Ok(State {
                stage: Stage::EntryEvents(chosen),
                ..state
            })
        }
        _ => Ok(state),
    }
}*/
