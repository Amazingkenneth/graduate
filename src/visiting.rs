use crate::{ChoosingState, Stage, State};

pub async fn get_queue(state: State) -> Result<State, crate::Error> {
    Ok(State {
        stage: Stage::ShowingPlots,
        ..state
    })
}
