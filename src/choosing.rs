use crate::{ChoosingState, State};
use iced::widget::image;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use toml::value::{Array, Table};

#[derive(Clone, Deserialize, Debug)]
pub struct Profile {
    nickname: Option<Array>,
    plots: Option<Array>,
    relationship: Option<Array>,
    comments: Option<Array>,
}

pub async fn get_configs(state: State) -> Result<State, crate::Error> {
    let has = state
        .idxtable
        .get("profile")
        .expect("Cannot get profile.")
        .as_table()
        .expect("Cannot read as an array")
        .to_owned();
    let mut names: Vec<String> = Vec::with_capacity(has.len());
    names.push(String::from("合照"));
    for value in has.values() {
        let mut name = value
            .as_str()
            .expect("Cannot convert into `String`")
            .to_string();
        name.pop();
        names.push(name.split_off(1));
    }
    let mut img_array: Vec<Option<image::Handle>> = Vec::new();
    img_array.resize(names.len(), Default::default());
    let profile_array: Vec<Profile> = Vec::new();
    img_array.resize(names.len(), Default::default());

    let img_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(img_array));
    let profile_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(profile_array));

    let url_prefix = state
        .idxtable
        .get("url_prefix")
        .expect("Cannot get the prefix")
        .as_str()
        .expect("cannot convert into str.")
        .to_string();
    let mut threads = vec![];
    fs::create_dir_all(Path::new(&format!("{}/profile", state.storage)))
        .expect("Cannot create the directory for profile.");
    fs::create_dir_all(Path::new(&format!("{}/image/known_people", state.storage)))
        .expect("Cannot create the directory for image.");
    for num in 1..=names.len() {
        let img_mutex = img_mutex.clone();
        let profile_mutex = profile_mutex.clone();
        let storage = state.storage.clone();
        let url_prefix = url_prefix.clone();

        let t = tokio::spawn(async move {
            let profile_path = Path::new(&format!("{}/profile/{}.toml", storage, num)).to_owned();
            let img_path =
                Path::new(&format!("{}/image/known_people/{}.jpg", storage, num)).to_owned();
            let profile_url = format!("{}/profile/{}.toml", url_prefix, num);
            let img_url = format!("{}/image/known_people/{}.jpg", url_prefix, num);
            let cli = Client::new();
            if img_path.is_file() {
                let mut img_mutex = img_mutex.lock().unwrap();
                img_mutex[num] = Some(image::Handle::from_path(&img_path));
            } else {
                let img_bytes = cli
                    .get(&img_url)
                    .send()
                    .await
                    .expect("Cannot send request")
                    .bytes()
                    .await
                    .expect("Cannot read the image into bytes.");
                let mut img_file =
                    std::fs::File::create(&img_path).expect("Failed to create image file.");
                img_file
                    .write_all(&img_bytes)
                    .expect("Failed to write the image into file in the project directory.");
            }
            if profile_path.is_file() {
                let profile_text =
                    fs::read_to_string(&profile_path).expect("Cannot read profile from file.");
                if let Ok(res) = toml::from_str(profile_text.as_str()) {
                    let mut profile_array = profile_mutex.lock().unwrap();
                    profile_array[num] = res;
                    return;
                }
            }
            let profile_text = cli
                .get(&profile_url)
                .send()
                .await
                .expect("Cannot send request")
                .text()
                .await
                .expect("Cannot read the image into bytes.");
            let mut profile_file =
                std::fs::File::create(&profile_path).expect("Failed to create profile file.");
            profile_file
                .write_all(&profile_text.as_bytes())
                .expect("Failed to write the image into file in the project directory.");
            let mut profile_array = profile_mutex.lock().unwrap();
            profile_array[num] =
                toml::from_str(profile_text.as_str()).expect("Cannot parse into `Profile` type.");
        });
        threads.push(t);
    }

    // 等待所有线程结束
    for t in threads {
        t.await?;
    }
    let img_fetched = img_mutex.lock().unwrap();
    let profile_fetched = profile_mutex.lock().unwrap();
    Ok(State {
        stage: crate::Stage::ChoosingCharacter(ChoosingState {
            photos: img_fetched.to_vec(),
            profiles: profile_fetched.to_vec(),
            on_character: None,
        }),
        ..state
    })
}
