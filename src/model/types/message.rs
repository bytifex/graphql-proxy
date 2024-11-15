use std::sync::Arc;

use async_graphql::Object;

use crate::model::enums::{connection_type::ConnectionType, message_direction::MessageDirection};

#[derive(Clone)]
pub struct Message {
    pub sequence_counter: u64,
    pub message: Arc<serde_json::Value>,
    pub connection_type: ConnectionType,
    pub message_direction: MessageDirection,
    pub connection_id: Arc<String>,
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
}
