use crate::{Error, Memories, Stage, State};
use iced::widget::image;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use toml::value::Value;
//type JoinHandle = std::thread::JoinHandle<_>;
impl State {
    pub async fn get_idx() -> Result<State, crate::Error> {
        let proj_dir = directories::ProjectDirs::from("", "9B1", "Graduate").unwrap();
        fs::create_dir_all(proj_dir.data_dir()).unwrap();
        let location = "https://graduate-1313398930.cos.ap-guangzhou.myqcloud.com";
        let idxurl: String = format!("{}{}", location, "/index.toml");
        let idxdir: String = format!("{}{}", proj_dir.data_dir().display(), "/index.toml");
        let mut contents = String::new();
        let cli = Client::new();
        if Path::new(&idxdir).is_file() {
            let mut file = std::fs::File::open(&idxdir).unwrap();
            file.read_to_string(&mut contents)
                .expect("Cannot read the index file");
            if let Ok(val) = contents.parse::<Value>() {
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
        let val = content.parse::<Value>().expect("Cannot parse the content.");
        Ok(State {
            stage: Stage::ChoosingCharacter(Default::default()),
            idxtable: val.as_table().ok_or(crate::Error::ParseError)?.to_owned(),
            client: cli,
            url_prefix: location.to_string(),
            storage: proj_dir.data_dir().to_str().unwrap().to_string(),
        })
    }
    pub async fn get_image(&self, path: String) -> Result<image::Handle, reqwest::Error> {
        let img_dir = format!("{}{}", self.storage, path);
        if Path::new(&img_dir).is_file() {
            return Ok(image::Handle::from_path(&img_dir));
        }
        async {
            let url = format!("{}{}", self.url_prefix, path);
            let bytes = self
                .client
                .get(&url)
                .send()
                .await
                .expect("Cannot send request")
                .bytes()
                .await
                .expect("Cannot read the image into bytes.");
            Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
        }
        .await
    }
}
