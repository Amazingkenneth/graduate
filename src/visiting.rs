use crate::{ChoosingState, Stage, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Event {
    pub description: Option<String>,
    pub date: time::PrimitiveDateTime,
    pub images: Vec<EventImage>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventImage {
    pub path: String,
    pub with: Vec<usize>,
}

pub async fn get_queue(state: State) -> Result<State, crate::Error> {
    let queue_list = state.idxtable.get("event").unwrap().as_array().unwrap();
    let mut performing = vec![];
    for event_table in queue_list {
        let cur_event: Event = event_table
            .as_table()
            .unwrap()
            .to_owned()
            .try_into()
            .unwrap();
        performing.push(cur_event);
    }
    Ok(State {
        stage: Stage::ShowingPlots(Default::default()),
        ..state
    })
}
