use crate::{Memories, Stage, State};
use iced::widget::image;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
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

impl Event {
    pub fn get_image_handle(&self) -> Option<image::Handle> {
        self.experiences[self.on_experience].handle.clone()
    }
    pub fn get_join_handle(&self) -> Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> {
        self.experiences[self.on_experience].join_handle.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Experience {
    pub shot: ShootingTime,
    pub path: String,
    pub handle: Option<image::Handle>,
    pub join_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl PartialEq for Experience {
    fn eq(&self, other: &Self) -> bool {
        self.shot == other.shot
    }
}
impl Eq for Experience {}
impl PartialOrd for Experience {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Experience {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use time::macros::time;
        let cmp_a = match self.shot {
            ShootingTime::Approximate(approximate) => {
                PrimitiveDateTime::new(approximate, time!(0:00))
            }
            ShootingTime::Precise(precise) => precise.clone(),
        };
        let cmp_b = match other.shot {
            ShootingTime::Approximate(approximate) => {
                PrimitiveDateTime::new(approximate, time!(0:00))
            }
            ShootingTime::Precise(precise) => precise.clone(),
        };
        cmp_a.cmp(&cmp_b)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
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
impl Into<toml::value::Datetime> for ShootingTime {
    fn into(self) -> toml::value::Datetime {
        match self {
            ShootingTime::Approximate(approximate) => toml::value::Datetime {
                date: Some(toml::value::Date {
                    day: approximate.day(),
                    month: approximate.month() as u8,
                    year: approximate.year() as u16,
                }),
                time: None,
                offset: None,
            },
            ShootingTime::Precise(precise) => toml::value::Datetime {
                date: Some(toml::value::Date {
                    day: precise.day(),
                    month: precise.month() as u8,
                    year: precise.year() as u16,
                }),
                time: Some(toml::value::Time {
                    hour: precise.hour(),
                    minute: precise.minute(),
                    second: precise.second(),
                    nanosecond: 0,
                }),
                offset: None,
            },
        }
    }
}
impl std::fmt::Display for ShootingTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShootingTime::Approximate(approximate) => std::fmt::Display::fmt(&approximate, f),
            ShootingTime::Precise(precise) => write!(
                f,
                "{} {}:{:02}:{:02}",
                precise.date(),
                precise.time().hour(),
                precise.time().minute(),
                precise.time().second(),
            ),
        }
    }
}

pub async fn get_queue(state: State) -> Result<State, crate::Error> {
    let events_path = std::path::Path::new(&format!("{}/events.toml", &state.storage)).to_owned();
    let queue_table = {
        if events_path.is_file() {
            let events_text = fs::read_to_string(&events_path).unwrap();
            toml::Table::from_str(events_text.as_str()).unwrap()
        } else {
            let events_url = format!(
                "{}/events.toml",
                state.idxtable.get("url_prefix").unwrap().as_str().unwrap()
            );
            let events_text = reqwest::get(events_url)
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            let mut events_file = std::fs::File::create(&events_path).unwrap();
            events_file.write_all(&events_text.as_bytes()).unwrap();
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
    let homepage_offset = match state.stage {
        Stage::ChoosingCharacter(choosing) => choosing.homepage_offset,
        _ => return Err(crate::Error::APIError),
    };
    let queue_array = queue_table.get("event").unwrap().as_array().unwrap();
    let experience_array = queue_table.get("experience").unwrap().as_array().unwrap();
    let mut rng = rand::thread_rng();
    for exp in experience_array {
        let cur_exp = exp.as_table().unwrap();
        let experience = cur_exp.get("image").unwrap().as_array().unwrap();
        let mut images = Vec::<Experience>::with_capacity(experience.len());
        for img in experience {
            let img_shotdate = img.get("date").unwrap().as_datetime().unwrap().into();
            let Some(with_people) = img.get("with") else {
                images.push(Experience {
                    path: img.get("path").unwrap().as_str().unwrap().to_string(),
                    shot: img_shotdate,
                    handle: None,
                    join_handle: Arc::new(Mutex::new(None)),
                });
                continue;
            };
            let with_people = with_people.as_array().unwrap();
            let mut with = Vec::with_capacity(with_people.len());
            for i in with_people {
                with.push(i.as_integer().unwrap() as usize);
            }
            if with.contains(&chose_person) {
                images.push(Experience {
                    path: format!(
                        "image/experience/{}",
                        img.get("path").unwrap().as_str().unwrap()
                    ),
                    shot: img_shotdate,
                    handle: None,
                    join_handle: Arc::new(Mutex::new(None)),
                });
                break;
            }
        }
        if !images.is_empty() {
            let description = cur_exp
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            for index in (0..images.len() - 1).rev() {
                if images[index] != images[index + 1] {
                    let mut shuffling = images.split_off(index + 1);
                    shuffling.shuffle(&mut rng);
                    queue_event.push(Event {
                        description: description.clone(),
                        experiences: shuffling,
                        on_experience: 0,
                    });
                }
            }
        }
    }
    for event in queue_array {
        let cur_table = event.as_table().unwrap();
        let experience = cur_table.get("image").unwrap().as_array().unwrap();
        let mut images = Vec::<Experience>::with_capacity(experience.len());
        for img in experience {
            let img_shotdate = img.get("date").unwrap().as_datetime().unwrap().into();
            let Some(with_people) = img.get("with") else {
                images.push(Experience {
                    path: img.get("path").unwrap().as_str().unwrap().to_string(),
                    shot: img_shotdate,
                    handle: None,
                    join_handle: Arc::new(Mutex::new(None)),
                });
                continue;
            };
            let with_people = with_people.as_array().unwrap();
            let mut with = Vec::with_capacity(with_people.len());
            for i in with_people {
                with.push(i.as_integer().unwrap() as usize);
            }
            if with.contains(&chose_person) {
                images.push(Experience {
                    path: img.get("path").unwrap().as_str().unwrap().to_string(),
                    shot: img_shotdate,
                    handle: None,
                    join_handle: Arc::new(Mutex::new(None)),
                });
                break;
            }
        }
        if !images.is_empty() {
            images.shuffle(&mut rng);
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
    }
    queue_event.sort_unstable();
    let initial_event = Event {
        description: String::from(""),
        on_experience: 0,
        experiences: vec![Experience {
            shot: state.configs.from_date.clone(),
            path: String::from(""),
            handle: None,
            join_handle: Arc::new(Mutex::new(None)),
        }],
    };
    let on_event = queue_event.partition_point(|event| event < &initial_event);
    fs::create_dir_all(format!("{}/image/experience", state.storage)).unwrap();
    fs::create_dir_all(format!("{}/image/camera", state.storage)).unwrap();
    let mut state = State {
        stage: Stage::ShowingPlots(crate::VisitingState {
            homepage_offset,
            character_name,
            events: Arc::new(Mutex::new(queue_event)),
            on_event,
        }),
        ..state
    };
    load_images(&mut state);
    Ok(state)
}

pub async fn force_load(
    join_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    memo: Memories,
) -> Result<Memories, crate::Error> {
    let a: Option<tokio::task::JoinHandle<()>> = std::mem::take(&mut join_handle.lock().unwrap());
    a.unwrap().await.unwrap();
    Ok(memo)
}

pub fn load_images(state: &mut State) {
    match state.stage {
        Stage::ShowingPlots(ref displayer) => {
            let len = {
                let events = displayer.events.lock().unwrap();
                events.len()
            };
            let location = state.idxtable.get("url_prefix").unwrap().as_str().unwrap();
            let left = {
                if displayer.on_event > 0 {
                    displayer.on_event - 1
                } else {
                    0
                }
            };
            for cur_idx in left..std::cmp::min(displayer.on_event + 5, len) {
                let mut events = displayer.events.lock().unwrap();
                for (cur_img, experience) in events[cur_idx].experiences.iter_mut().enumerate() {
                    let need_to_load = match *experience.join_handle.lock().unwrap() {
                        None => true,
                        _ => false,
                    };
                    if need_to_load {
                        // let path = experience.path;
                        let img_dir = format!("{}{}", &state.storage, experience.path);
                        let img_path = std::path::Path::new(&img_dir);
                        if img_path.is_file() {
                            experience.handle = Some(image::Handle::from_path(&img_dir));
                            continue;
                        }
                        let url = format!("{}{}", location, experience.path);
                        let given_mutex = displayer.events.clone();
                        let t = tokio::spawn(async move {
                            let bytes = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
                            let mut file = std::fs::File::create(&img_dir).unwrap();
                            file.write_all(&bytes).unwrap();
                            given_mutex.lock().unwrap()[cur_idx].experiences[cur_img].handle =
                                Some(image::Handle::from_memory(bytes));
                        });
                        *experience.join_handle.lock().unwrap() = Some(t);
                    }
                }
            }
        }
        _ => (),
    }
}
