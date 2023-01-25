use crate::{ChoosingState, Stage, State};
use iced::event;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::str::FromStr;
use time::{Date, PrimitiveDateTime};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct Event {
    pub description: String,
    pub images: Vec<Experience>,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.images
            .first()
            .unwrap()
            .cmp(other.images.first().unwrap())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct Experience {
    pub shot: ShootingTime,
    pub path: String,
    pub with: Option<Vec<usize>>,
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
    let queue_array = queue_table.get("event").unwrap().as_array().unwrap();
    let mut queue_event = Vec::<Event>::with_capacity(queue_table.len());
    for event in queue_array {
        let cur_table = event.as_table().unwrap();
        let experience = cur_table.get("image").unwrap().as_array().unwrap();
        let mut images = Vec::<Experience>::with_capacity(experience.len());
        for img in experience {
            let img_shotdate = img.get("date").unwrap().as_datetime().unwrap().into();
            let mut res_with = None;
            if let Some(with_people) = img.get("with") {
                let with_people = with_people.as_array().unwrap();
                let mut with = Vec::with_capacity(with_people.len());
                for i in with_people {
                    with.push(i.as_integer().unwrap() as usize);
                }
                res_with = Some(with);
            }
            images.push(Experience {
                path: img.get("path").unwrap().as_str().unwrap().to_string(),
                shot: img_shotdate,
                with: res_with,
            });
        }
        queue_event.push(Event {
            description: cur_table
                .get("description")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            images,
        });
    }
    match state.stage {
        Stage::ChoosingCharacter(choosing) => {
            let profiles = choosing.profiles;
            let chose_person = choosing.on_character.unwrap();
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
                            let mut with: Vec<usize> = vec![5];
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
                                    });
                                    break;
                                }
                            }
                            // with.sort_unstable();
                            // if with.binary_search(choosing.on_character.unwrap()) {}
                        }
                        queue_event.push(Event {
                            description: event_table
                                .get("description")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                            images: personal_images,
                        });
                    }
                }
            }
        }
        _ => return Err(crate::Error::APIError),
    }
    queue_event.sort_unstable();
    Ok(State {
        stage: Stage::ShowingPlots(Default::default()),
        ..state
    })
}
