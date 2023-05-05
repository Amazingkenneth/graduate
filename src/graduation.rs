use crate::{ChoosingState, Stage, State};
use iced::widget::image;
use reqwest::Client;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use std::sync::atomic::{AtomicUsize, Ordering};
pub static ON_LOCATION: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, Default)]
pub struct Panorama {
    pub image: Vec<image::Handle>,
    pub image_names: Vec<String>,
    pub pinpoint: (f32, f32),
}

pub async fn load_map(state: State) -> Result<State, crate::Error> {
    fs::create_dir_all(format!("{}/image/panorama", state.storage)).unwrap();
    let panoramas = state
        .idxtable
        .get("panorama")
        .unwrap()
        .as_array()
        .unwrap()
        .to_owned();
    let location = state
        .idxtable
        .get("url_prefix")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let mut images: Vec<Vec<image::Handle>> = Vec::new();
    images.resize(panoramas.len(), vec![]);
    let img_mutex: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(images));
    let mut threads = vec![];
    for (i, cur_image) in panoramas.iter().enumerate() {
        let fetching: Vec<toml::Value> = cur_image
            .get("image")
            .unwrap()
            .as_array()
            .unwrap()
            .to_owned();
        let img_mutex = img_mutex.clone();
        let storage = state.storage.clone();
        let location = location.clone();
        let t = tokio::spawn(async move {
            let cli = Client::new();
            let mut fillin: Vec<image::Handle> = Vec::new();
            for cur_image in fetching {
                let mut img_path_string = cur_image.to_owned().to_string();
                img_path_string.pop();
                let relative_path = img_path_string.split_off(1);
                let img_dir = format!("{}/image/panorama/{}", &storage, relative_path);
                let img_path = Path::new(&img_dir);
                if img_path.is_file() {
                    fillin.push(image::Handle::from_path(&img_dir));
                    continue;
                }
                fs::create_dir_all(img_path.parent().unwrap()).unwrap();
                let url = format!("{}/image/panorama/{}", location, relative_path);
                let bytes = cli.get(&url).send().await.unwrap().bytes().await.unwrap();
                let mut file = std::fs::File::create(&img_dir).unwrap();
                file.write_all(&bytes).unwrap();
                fillin.push(image::Handle::from_memory(bytes));
            }
            let mut images = img_mutex.lock().unwrap();
            images[i] = fillin;
        });
        threads.push(t);
    }
    for t in threads {
        t.await?;
    }
    let img_fetched = img_mutex.lock().unwrap().to_vec();
    let mut pans: Vec<Panorama> = Vec::with_capacity(panoramas.len());
    for (i, pan) in panoramas.iter().enumerate() {
        let fetching: Vec<toml::Value> = pan.get("image").unwrap().as_array().unwrap().to_owned();
        let mut names = vec![];
        for j in fetching {
            let name = j
                .as_str()
                .unwrap()
                .strip_suffix(".jpg")
                .unwrap()
                .to_string();
            names.push(name);
        }
        let point = pan.get("pinpoint").unwrap().as_array().unwrap();
        let x = point[0].as_integer().unwrap() as f32;
        let y = point[1].as_integer().unwrap() as f32;
        dbg!(&names);
        pans.push(Panorama {
            image: img_fetched[i].clone(),
            image_names: names,
            pinpoint: (x, y),
        });
    }
    Ok(State {
        stage: Stage::Graduated(crate::GraduationState {
            show_panel: true,
            on_image: 0,
            images: pans,
        }),
        ..state
    })
}
