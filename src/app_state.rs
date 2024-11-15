use std::sync::Arc;

use crate::control_state::ControlState;

struct AppStateInner {
    server_client: reqwest::Client,
}

#[derive(Clone)]
pub struct AppState {
    params: Arc<AppStateInner>,
    control_state: ControlState,
}

impl AppState {
    pub fn new(control_state: ControlState) -> Result<Self, reqwest::Error> {
        Ok(Self {
            params: Arc::new(AppStateInner {
                server_client: reqwest::ClientBuilder::new().build()?,
            }),
            control_state,
        })
    }

    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    pub fn server_client(&self) -> &reqwest::Client {
        &self.params.server_client
    }
}
