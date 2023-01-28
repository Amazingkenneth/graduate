use crate::{ChoosingState, Stage, State};
use iced::event;
use iced::widget::image;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{error::Error, fs};
use time::{Date, PrimitiveDateTime};

#[derive(Clone, Debug)]
pub struct Event {
    pub description: String,
    pub on_experience: usize,
    pub experiences: Vec<Experience>,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.description == other.description
    }
}
impl Eq for Event {}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use time::macros::time;
        if self.experiences.len() == 0 {
            println!("Error processing: {:?}", self);
        }
        let cmp_a = match self.experiences.first().unwrap().shot {
            ShootingTime::Approximate(approximate) => {
                PrimitiveDateTime::new(approximate, time!(0:00))
            }
            ShootingTime::Precise(precise) => precise.clone(),
        };
        let cmp_b = match other.experiences.first().unwrap().shot {
            ShootingTime::Approximate(approximate) => {
                PrimitiveDateTime::new(approximate, time!(0:00))
            }
            ShootingTime::Precise(precise) => precise.clone(),
        };
        cmp_a.cmp(&cmp_b)
    }
}

#[derive(Clone, Debug)]
pub struct Experience {
    pub shot: ShootingTime,
    pub path: String,
    pub with: Option<Vec<usize>>,
    pub handle: Option<image::Handle>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub enum ShootingTime {
    Precise(time::PrimitiveDateTime),
    Approximate(time::Date),
}

const PRECISE_FORMAT: &[time::format_description::FormatItem] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
const APPROXIMATE_FORMAT: &[time::format_description::FormatItem] =
    time::macros::format_description!("[year]-[month]-[day]");

impl From<&toml::value::Datetime> for ShootingTime {
    fn from(item: &toml::value::Datetime) -> Self {
        if let Ok(res) = Date::parse(&item.to_string(), &APPROXIMATE_FORMAT) {
            return ShootingTime::Approximate(res);
        } else {
            return ShootingTime::Precise(
                PrimitiveDateTime::parse(&item.to_string(), &PRECISE_FORMAT).unwrap(),
            );
        }
    }
}
impl std::fmt::Display for ShootingTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShootingTime::Approximate(approximate) => std::fmt::Display::fmt(&approximate, f),
            ShootingTime::Precise(precise) => std::fmt::Display::fmt(&precise, f),
        }
    }
}

pub async fn get_queue(state: State) -> Result<State, crate::Error> {
    let events_path = std::path::Path::new(&format!("{}/events.toml", &state.storage)).to_owned();
    let queue_table = {
        if events_path.is_file() {
            let events_text =
                fs::read_to_string(&events_path).expect("Cannot read events from file.");
            toml::Table::from_str(events_text.as_str()).unwrap()
        } else {
            let events_url = format!(
                "{}/events.toml",
                state.idxtable.get("url_prefix").unwrap().as_str().unwrap()
            );
            let events_text = reqwest::get(events_url)
                .await
                .expect("Cannot send request")
                .text()
                .await
                .unwrap();
            let mut events_file =
                std::fs::File::create(&events_path).expect("Failed to create events file.");
            events_file
                .write_all(&events_text.as_bytes())
                .expect("Failed to write the image into file in the project directory.");
            toml::Table::from_str(events_text.as_str()).unwrap()
        }
    };
    let mut queue_event = Vec::<Event>::with_capacity(queue_table.len());
    let (chose_person, character_name) = match state.stage {
        Stage::ChoosingCharacter(ref choosing) => {
            let on = choosing.on_character.unwrap();
            (on, choosing.avatars[on].name.clone())
        }
        _ => (0, String::from("")),
    };
    match state.stage {
        Stage::ChoosingCharacter(choosing) => {
            let profiles = choosing.profiles;
            for (num, profile) in profiles.iter().enumerate() {
                if let Some(experience) = &profile.experience {
                    for event_value in experience {
                        let event_table = event_value.as_table().unwrap();
                        let mut event_time = None;
                        if let Some(occur_time) = event_table.get("date") {
                            event_time = occur_time.as_datetime().unwrap().into();
                        }
                        let img_array = event_table.get("image").unwrap().as_array().unwrap();
                        let mut personal_images = vec![];
                        for img in img_array {
                            let shot = if let Some(meta_date) = img.get("date") {
                                meta_date.as_datetime().unwrap().into()
                            } else {
                                event_time.unwrap().into()
                            };
                            let mut with: Vec<usize> = vec![num];
                            if let Some(appearing) = img.get("with") {
                                with = appearing
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|v| v.as_integer().unwrap() as usize)
                                    .collect();
                                with.push(num as usize);
                            }
                            for it in &with {
                                if it == &chose_person {
                                    personal_images.push(Experience {
                                        path: img
                                            .get("path")
                                            .unwrap()
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        shot,
                                        with: Some(with),
                                        handle: None,
                                    });
                                    break;
                                }
                            }
                        }
                        if !personal_images.is_empty() {
                            queue_event.push(Event {
                                description: event_table
                                    .get("description")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                experiences: personal_images,
                                on_experience: 0,
                            });
                        }
                    }
                }
            }
        }
        _ => return Err(crate::Error::APIError),
    }
    let queue_array = queue_table.get("event").unwrap().as_array().unwrap();
    for event in queue_array {
        let cur_table = event.as_table().unwrap();
        let experience = cur_table.get("image").unwrap().as_array().unwrap();
        let mut images = Vec::<Experience>::with_capacity(experience.len());
        for img in experience {
            let img_shotdate = img.get("date").unwrap().as_datetime().unwrap().into();
            if let Some(with_people) = img.get("with") {
                let with_people = with_people.as_array().unwrap();
                let mut with = Vec::with_capacity(with_people.len());
                for i in with_people {
                    with.push(i.as_integer().unwrap() as usize);
                }
                for it in &with {
                    if it == &chose_person {
                        images.push(Experience {
                            path: img.get("path").unwrap().as_str().unwrap().to_string(),
                            shot: img_shotdate,
                            with: Some(with),
                            handle: None,
                        });
                        break;
                    }
                }
            } else {
                images.push(Experience {
                    path: img.get("path").unwrap().as_str().unwrap().to_string(),
                    shot: img_shotdate,
                    with: None,
                    handle: None,
                });
            }
        }
        queue_event.push(Event {
            description: cur_table
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            experiences: images,
            on_experience: 0,
        });
    }
    // println!("{:?}", queue_event);
    queue_event.sort_unstable();
    let on_event = queue_event
        .partition_point(|event| event.experiences.first().unwrap().shot < state.configs.from_date);
    fs::create_dir_all(format!("{}/image/experience", state.storage)).unwrap();
    fs::create_dir_all(format!("{}/image/camera", state.storage)).unwrap();

    load_images(State {
        stage: Stage::ShowingPlots(crate::VisitingState {
            character_name,
            events: Arc::new(Mutex::new(queue_event)),
            on_event,
        }),
        ..state
    })
    .await
}

pub async fn load_images(state: State) -> Result<State, crate::Error> {
    let mut threads = vec![];
    match state.stage {
        Stage::ShowingPlots(ref displayer) => {
            let mut events = displayer.events.lock().unwrap();
            let location = state.idxtable.get("url_prefix").unwrap().as_str().unwrap();
            for cur_idx in displayer.on_event..std::cmp::min(displayer.on_event + 5, events.len()) {
                for (cur_img, experience) in events[cur_idx].experiences.iter_mut().enumerate() {
                    if let None = experience.handle {
                        let given_mutex = displayer.events.clone();
                        // let path = experience.path;
                        let img_dir = format!("{}{}", &state.storage, experience.path);
                        let img_path = std::path::Path::new(&img_dir);
                        if img_path.is_file() {
                            experience.handle = Some(image::Handle::from_path(&img_dir));
                            continue;
                        }
                        let url = format!("{}{}", location, experience.path);
                        threads.push(tokio::spawn(async move {
                            let bytes = reqwest::get(&url)
                                .await
                                .expect("Cannot send request")
                                .bytes()
                                .await
                                .expect("Cannot read the image into bytes.");
                            println!("Done processing image!");
                            let mut file = std::fs::File::create(&img_dir)
                                .expect("Failed to create image file.");
                            file.write_all(&bytes).expect(
                                "Failed to write the image into file in the project directory.",
                            );
                            let mut events = given_mutex.lock().unwrap();
                            events[cur_idx].experiences[cur_img].handle =
                                Some(image::Handle::from_memory(bytes.as_ref().to_vec()));
                        }));
                    }
                }
            }
        }
        _ => return Err(crate::Error::APIError),
    }
    for i in threads {
        i.await?;
    }
    Ok(state)
}
