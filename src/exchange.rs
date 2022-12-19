use crate::{Error, Memories, Stage, State};
use iced::widget::image;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use toml::value;

//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx() -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "9B1", "Graduate").unwrap();
        fs::create_dir_all(proj_dir.data_dir()).unwrap();
        let location = "https://graduate-1313398930.cos.ap-guangzhou.myqcloud.com";
        let idxurl: String = format!("https://amazingkenneth.github.io/graduate/index.toml");
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let mut contents = String::new();
        let cli = Client::new();
        if Path::new(&idxdir).is_file() {
            let mut file = std::fs::File::open(&idxdir).unwrap();
            file.read_to_string(&mut contents)
                .expect("Cannot read the index file");
            if let Ok(val) = contents.parse::<toml::value::Value>() {
                println!("Successfully parsed content.");
                return Ok(State {
                    stage: Stage::ChoosingCharacter(Default::default()),
                    idxtable: val.as_table().ok_or(crate::Error::ParseError)?.to_owned(),
                    client: cli,
                    url_prefix: location.to_string(),
                    storage: proj_dir.data_dir().to_str().unwrap().to_string(),
                });
            }
        }
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
        let val = content
            .parse::<toml::value::Value>()
            .expect("Cannot parse the content.");
        Ok(State {
            stage: Stage::ChoosingCharacter(Default::default()),
            idxtable: val.as_table().ok_or(crate::Error::ParseError)?.to_owned(),
            client: cli,
            url_prefix: location.to_string(),
            storage: proj_dir.data_dir().to_str().unwrap().to_string(),
        })
    }
    pub async fn get_image(&self, path: String) -> Result<image::Handle, reqwest::Error> {
        println!("Calling get_image({})", path);
        let img_dir = format!("{}{}", self.storage, path);
        println!("Calling get_image({})", img_dir);
        let img_path = Path::new(&img_dir);
        if img_path.is_file() {
            return Ok(image::Handle::from_path(&img_dir));
        }
        fs::create_dir_all(img_path.parent().expect("Cannot parse the path.")).unwrap();
        async {
            let url = format!("{}{}", self.url_prefix, path);
            println!("url: {}", url);
            let bytes = self
                .client
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
    pub fn get_current_event(&self, on_event: u32) -> toml::value::Value {
        self.idxtable
            .get("event")
            .expect("Cannot get the `event` array.")
            .as_array()
            .expect("Cannot read as an array.")[on_event as usize]
            .to_owned()
    }
}
pub async fn change_image(state: State, mut to: i64) -> Result<State, crate::Error> {
    match state.stage {
        Stage::ChoosingCharacter(ref chosen) => {
            let cnt = state
                .idxtable
                .get("together_events")
                .expect("Didn't find together_events in the indextable.")
                .as_integer()
                .expect("together_events is not an integer");
            let mut chosen = chosen.clone();
            if to >= cnt {
                to = 0;
            }
            if to < 0 {
                to = cnt - 1;
            }
            chosen.on_event = to as u32;
            let mut img_path = state
                .get_current_event(chosen.on_event)
                .get("image")
                .expect("No image value in the item.")
                .as_array()
                .expect("Cannot read the path.")[chosen.on_image as usize]
                .to_owned()
                .to_string();
            img_path.pop();
            chosen.image = Some(
                state
                    .get_image(img_path.split_off(1))
                    .await
                    .expect("Cannot get image."),
            );
            Ok(State {
                stage: Stage::ChoosingCharacter(chosen),
                ..state
            })
        }
        _ => Ok(state),
    }
}

pub async fn load_image(state: State) -> Result<State, crate::Error> {
    match state.stage {
        Stage::ChoosingCharacter(ref chosen) => {
            let mut chosen = chosen.clone();
            let mut img_path = state
                .get_current_event(chosen.on_event)
                .get("image")
                .expect("No image value in the item.")
                .as_array()
                .expect("Cannot read the path.")[chosen.on_image as usize]
                .to_owned()
                .to_string();
            img_path.pop();
            chosen.image = Some(
                state
                    .get_image(img_path.split_off(1))
                    .await
                    .expect("Cannot get image."),
            );
            Ok(State {
                stage: Stage::ChoosingCharacter(chosen),
                ..state
            })
        }
        _ => Ok(state),
    }
}
