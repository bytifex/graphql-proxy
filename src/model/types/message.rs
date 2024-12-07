use std::sync::Arc;

use async_graphql::Object;

use crate::model::enums::{connection_type::ConnectionType, message_direction::MessageDirection};

use super::headers::Headers;

#[derive(Clone)]
pub struct Message {
    pub sequence_counter: u64,
    pub message: Arc<serde_json::Value>,
    pub connection_type: ConnectionType,
    pub message_direction: MessageDirection,
    pub connection_id: Arc<String>,
    pub transmitted_headers: Option<Arc<Headers>>,
    pub server_endpoint_url: Arc<String>,
}

#[Object]
impl Message {
    async fn sequence_counter(&self) -> u64 {
        self.sequence_counter
    }

    async fn message(&self) -> &serde_json::Value {
        &self.message
    }

    async fn connection_type(&self) -> ConnectionType {
        self.connection_type
    }

    async fn message_direction(&self) -> MessageDirection {
        self.message_direction
    }

    async fn connection_id(&self) -> &String {
        &self.connection_id
    }

    async fn transmitted_headers(&self) -> &Option<Arc<Headers>> {
        &self.transmitted_headers
    }

    async fn server_endpoint_url(&self) -> &String {
        &self.server_endpoint_url
    }
}
