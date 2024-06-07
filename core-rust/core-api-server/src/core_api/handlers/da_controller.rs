use crate::core_api::*;

// TODO yeah, it all sucks, it's just for demo purposes

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_da_status(
    state: State<CoreApiState>,
) -> Result<Json<DaStatusResponse>, ResponseError<()>> {
    let da_state = state.da_state.lock().unwrap();

    Ok(Json(DaStatusResponse {
        status: format!("all good from status, running = {}, start counter = {}", da_state.should_run, da_state.counter),
    }))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_da_start(
    state: State<CoreApiState>,
) -> Result<Json<DaStatusResponse>, ResponseError<()>> {
    let mut da_state = state.da_state.lock().unwrap();
    let res = if da_state.should_run {
        "already running"
    } else {
        da_state.should_run = true;
        da_state.counter += 1;

        "started"
    };

    Ok(Json(DaStatusResponse {
        status: format!("all good from start: {}", res),
    }))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn handle_da_stop(
    state: State<CoreApiState>,
) -> Result<Json<DaStatusResponse>, ResponseError<()>> {
    let mut da_state = state.da_state.lock().unwrap();
    let res = if da_state.should_run {
        da_state.should_run = false;
        "stopped"
    } else {
        "not running"
    };

    Ok(Json(DaStatusResponse {
        status: format!("all good from stop: {}", res),
    }))
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct DaStatusResponse {
    #[serde(rename = "status")]
    pub status: String,
}