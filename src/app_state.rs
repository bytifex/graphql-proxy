use std::sync::Arc;

use crate::admin_state::AdminState;

struct AppStateInner {
    server_client: reqwest::Client,
}

#[derive(Clone)]
pub struct AppState {
    params: Arc<AppStateInner>,
    admin_state: AdminState,
}

impl AppState {
    pub fn new(admin_state: AdminState) -> Result<Self, reqwest::Error> {
        Ok(Self {
            params: Arc::new(AppStateInner {
                server_client: reqwest::ClientBuilder::new().build()?,
            }),
            admin_state,
        })
    }

    pub fn admin_state(&self) -> &AdminState {
        &self.admin_state
    }

    pub fn server_client(&self) -> &reqwest::Client {
        &self.params.server_client
    }
}
