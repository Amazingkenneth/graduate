use crate::{ChoosingState, State};
use iced::widget::image;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use toml::value::{Array, Table};

#[derive(Clone, Default, Deserialize, Debug)]
pub struct Profile {
    pub nickname: Option<Array>,
    pub plots: Option<Array>,
    pub relationship: Option<Array>,
    pub comment: Option<Array>,
    pub name_en: Option<String>,
    pub introduction: Option<String>,
    pub experience: Option<Table>,
}

#[derive(Clone, Debug)]
pub struct Avatar {
    pub name: String,
    pub photo: image::Handle,
    pub emoji: Vec<Emoji>,
    pub shown: bool,
}

#[derive(Clone, Debug)]
pub struct Emoji {
    pub emoji: image::Handle,
    pub emoji_name: String,
}

pub async fn get_configs(state: State) -> Result<State, crate::Error> {
    let has = state
        .idxtable
        .get("profile")
        .expect("Cannot get profile.")
        .as_table()
        .expect("Cannot read as toml::table")
        .to_owned();
    let mut names: Vec<String> = Vec::with_capacity(has.len() + 1);
    names.push(String::from(""));
    for value in 1..=has.len() {
        names.push(
            has.get(&value.to_string())
                .expect("Cannot find the name of the given number")
                .as_str()
                .expect("Cannot convert into `String`")
                .to_string(),
        );
    }
    let mut img_array: Vec<Option<image::Handle>> = Vec::new();
    img_array.resize(names.len() + 1, Default::default());
    let mut profile_array: Vec<Profile> = Vec::new();
    profile_array.resize(names.len() + 1, Default::default());
    let mut emoji_array: Vec<Vec<Emoji>> = Vec::new();
    emoji_array.resize(names.len() + 1, Default::default());

    let img_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(img_array));
    let profile_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(profile_array));
    let emoji_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(emoji_array));

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

    for num in 1..names.len() {
        let img_mutex = img_mutex.clone();
        let profile_mutex = profile_mutex.clone();
        let storage = state.storage.clone();
        let url_prefix = url_prefix.clone();
        fs::create_dir_all(Path::new(&format!("{}/image/emoji/{}", state.storage, num)))
            .expect("Cannot create the directory for emoji.");

        let t = tokio::spawn(async move {
            let profile_path = Path::new(&format!("{}/profile/{}.toml", storage, num)).to_owned();
            let img_path =
                Path::new(&format!("{}/image/known_people/{}.jpg", storage, num)).to_owned();
            let profile_url = format!("{}/profile/{}.toml", url_prefix, num);
            let img_url = format!("{}/image/known_people/{}.jpg", url_prefix, num);
            let cli = Client::new();
            if img_path.is_file() {
                let mut img_array = img_mutex.lock().unwrap();
                img_array[num] = Some(image::Handle::from_path(&img_path));
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
                let mut img_array = img_mutex.lock().unwrap();
                img_array[num] = Some(image::Handle::from_memory(img_bytes.to_vec()));
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

    let emojis = state
        .idxtable
        .get("emoji")
        .expect("Cannot get emoji array!")
        .as_array()
        .expect("Cannot convert emojis into an array");
    for emoji in emojis {
        let cur_path = emoji
            .as_str()
            .expect("Cannot convert emoji item into str")
            .to_string();
        let emoji_url = format!("{}/image/emoji/{}", url_prefix, cur_path);
        let emoji_dir = format!("{}/image/emoji/{}", state.storage, cur_path);
        let emoji_mutex = emoji_mutex.clone();
        let t = tokio::spawn(async move {
            let (num_str, emoji_name) =
                cur_path.split_at(cur_path.find('/').expect("Not valid separator") + 1);
            let mut num_string = num_str.to_string();
            num_string.pop();
            println!("num_string = \"{}\"\n", num_string);
            let num = num_string.parse::<usize>().expect("Cannot parse the num");
            let emoji_path = Path::new(&emoji_dir);
            if emoji_path.is_file() {
                let mut emoji_array = emoji_mutex.lock().unwrap();
                emoji_array[num].push(Emoji {
                    emoji_name: emoji_name.to_string(),
                    emoji: image::Handle::from_path(&emoji_path),
                });
            } else {
                let emoji_bytes = reqwest::get(&emoji_url)
                    .await
                    .expect("Cannot send request")
                    .bytes()
                    .await
                    .expect("Cannot read the emoji into bytes.");
                let mut emoji_file =
                    std::fs::File::create(&emoji_path).expect("Failed to create image file.");
                emoji_file
                    .write_all(&emoji_bytes)
                    .expect("Failed to write the image into file in the project directory.");
                let mut emoji_array = emoji_mutex.lock().unwrap();
                emoji_array[num].push(Emoji {
                    emoji_name: emoji_name.to_string(),
                    emoji: image::Handle::from_memory(emoji_bytes.to_vec()),
                });
            }
        });
        threads.push(t);
    }

    // 等待所有线程结束
    for t in threads {
        t.await?;
    }
    let img_fetched = img_mutex.lock().unwrap().to_vec();
    let profile_fetched = profile_mutex.lock().unwrap();
    let emoji_fetched = emoji_mutex.lock().unwrap();
    let mut avatars: Vec<Avatar> = Vec::with_capacity(img_fetched.len() + 1);
    avatars.push(Avatar {
        name: String::from(""),
        photo: image::Handle::from_memory(vec![]),
        emoji: Vec::new(),
        shown: false,
    });
    for (index, value) in img_fetched.iter().enumerate() {
        if let Some(img) = &value {
            avatars.push(Avatar {
                name: names[index].to_owned(),
                photo: img.to_owned(),
                emoji: emoji_fetched[index].to_owned(),
                shown: true,
            });
        }
    }
    let mut rng = rand::thread_rng();
    let element_count: usize = rng.gen_range(6..=8);
    if let crate::Stage::EntryEvents(previous) = state.stage {
        Ok(State {
            stage: crate::Stage::ChoosingCharacter(ChoosingState {
                avatars,
                element_count,
                profiles: profile_fetched.to_vec(),
                on_character: None,
                description: String::from(""),
                previous_stage: previous,
            }),
            ..state
        })
    } else {
        panic!("State is not EntryState!")
    }
}

pub fn generate_id(i: usize) -> iced::widget::scrollable::Id {
    iced::widget::scrollable::Id::new(format!("ChoosingCharacter-{}", i))
}
