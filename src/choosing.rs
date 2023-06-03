use crate::{ChoosingState, State};
use iced::widget::image;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use toml::value::{Array, Table};

pub const CHARACTERS_WITH_NO_PHOTOS: [usize; 10] = [38, 43, 44, 45, 46, 47, 49, 50, 53, 54];
pub const SEMESTER_NAMES: [&str; 5] = [
    "七年级上学期",
    "七年级下学期",
    "八年级上学期",
    "八年级下学期",
    "九年级上学期",
];

#[derive(Clone, Default, Deserialize, Debug)]
pub struct Profile {
    pub anecdote: Option<Table>,
    pub article: Option<Array>,
    pub comment: Option<Array>,
    pub experience: Option<Array>,
    pub introduction: Option<String>,
    pub name_en: Option<String>,
    pub nickname: Option<Array>,
    pub plots: Option<Array>,
    pub relationship: Option<Array>,
    pub reviews: Option<Array>,
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

pub async fn get_configs(
    on_character: Option<usize>,
    homepage_offset: iced::widget::scrollable::RelativeOffset,
    state: State,
) -> Result<State, crate::Error> {
    let has = state
        .idxtable
        .get("profile")
        .unwrap()
        .as_table()
        .unwrap()
        .to_owned();
    let mut names: Vec<String> = Vec::with_capacity(has.len() + 1);
    names.push(String::from(""));
    for value in 1..=has.len() {
        names.push(
            has.get(&value.to_string())
                .unwrap()
                .as_str()
                .unwrap()
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
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let mut threads = vec![];
    fs::create_dir_all(Path::new(&format!("{}/profile", state.storage))).unwrap();
    fs::create_dir_all(Path::new(&format!("{}/image/known_people", state.storage))).unwrap();

    for num in 1..names.len() {
        let img_mutex = img_mutex.clone();
        let profile_mutex = profile_mutex.clone();
        let storage = state.storage.clone();
        let url_prefix = url_prefix.clone();
        fs::create_dir_all(Path::new(&format!("{}/image/emoji/{}", state.storage, num))).unwrap();

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
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap();
                let mut img_file = std::fs::File::create(&img_path).unwrap();
                img_file.write_all(&img_bytes).unwrap();
                let mut img_array = img_mutex.lock().unwrap();
                img_array[num] = Some(image::Handle::from_memory(img_bytes));
            }
            if profile_path.is_file() {
                let profile_text = fs::read_to_string(&profile_path).unwrap();
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
                .unwrap()
                .text()
                .await
                .unwrap();
            let mut profile_file = std::fs::File::create(&profile_path).unwrap();
            profile_file.write_all(&profile_text.as_bytes()).unwrap();
            let mut profile_array = profile_mutex.lock().unwrap();
            profile_array[num] = toml::from_str(profile_text.as_str()).unwrap();
        });
        threads.push(t);
    }

    let emojis = state.idxtable.get("emoji").unwrap().as_array().unwrap();
    for emoji in emojis {
        let cur_path = emoji.as_str().unwrap().to_string();
        let emoji_url = format!("{}/image/emoji/{}", url_prefix, cur_path);
        let emoji_dir = format!("{}/image/emoji/{}", state.storage, cur_path);
        let emoji_mutex = emoji_mutex.clone();
        let t = tokio::spawn(async move {
            let (num_str, emoji_name) = cur_path.split_at(cur_path.find('/').unwrap() + 1);
            let (mut num_string, mut emoji_name) = (num_str.to_string(), emoji_name.to_string());
            num_string.pop();
            emoji_name.truncate(emoji_name.find('.').unwrap());
            let num = num_string.parse::<usize>().unwrap();
            let emoji_path = Path::new(&emoji_dir);
            if emoji_path.is_file() {
                let mut emoji_array = emoji_mutex.lock().unwrap();
                emoji_array[num].push(Emoji {
                    emoji_name,
                    emoji: image::Handle::from_path(&emoji_path),
                });
            } else {
                let emoji_bytes = reqwest::get(&emoji_url)
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap();
                let mut emoji_file = std::fs::File::create(&emoji_path).unwrap();
                emoji_file.write_all(&emoji_bytes).unwrap();
                let mut emoji_array = emoji_mutex.lock().unwrap();
                emoji_array[num].push(Emoji {
                    emoji_name,
                    emoji: image::Handle::from_memory(emoji_bytes),
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
    let previous_stage = if let crate::Stage::EntryEvents(previous) = state.stage {
        Some(previous)
    } else {
        None
    };
    Ok(State {
        stage: crate::Stage::ChoosingCharacter(ChoosingState {
            avatars,
            element_count,
            on_character,
            profiles: profile_fetched.to_vec(),
            description: String::from(""),
            previous_stage,
            homepage_offset,
        }),
        ..state
    })
}

pub fn generate_scrollable_id(i: usize) -> iced::widget::scrollable::Id {
    iced::widget::scrollable::Id::new(format!("ChoosingCharacter-{}", i))
}
