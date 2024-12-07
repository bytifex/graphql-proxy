use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use http::HeaderMap;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::sync::broadcast;

use crate::model::types::{graphql_endpoints::GraphQLEndpoints, message::Message};

#[derive(Debug, Clone)]
pub struct ConnectionId(Arc<String>);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Arc::new(uuid::Uuid::new_v4().as_hyphenated().to_string()))
    }

    pub fn as_arc_string(&self) -> Arc<String> {
        self.0.clone()
    }
}

struct AdminStateInner {
    message_sender: broadcast::Sender<Message>,
    prohibit_mutation: AtomicBool,
    server_endpoints: RwLock<GraphQLEndpoints>,
    request_headers: Arc<RwLock<HeaderMap>>,
    response_headers: Arc<RwLock<HeaderMap>>,
}

#[derive(Clone)]
pub struct AdminState(Arc<AdminStateInner>);

impl AdminState {
    pub fn new(
        server_graphql_endpoint: impl Into<String>,
        server_graphql_ws_endpoint: impl Into<String>,
        prohibit_mutation: bool,
        request_headers: HeaderMap,
        response_headers: HeaderMap,
    ) -> Self {
        Self(Arc::new(AdminStateInner {
            message_sender: broadcast::channel(128).0,
            prohibit_mutation: AtomicBool::new(prohibit_mutation),
            server_endpoints: RwLock::new(GraphQLEndpoints {
                graphql_endpoint: server_graphql_endpoint.into(),
                graphql_ws_endpoint: server_graphql_ws_endpoint.into(),
            }),
            request_headers: Arc::new(RwLock::new(request_headers)),
            response_headers: Arc::new(RwLock::new(response_headers)),
        }))
    }

    pub fn message_sender_ref(&self) -> &broadcast::Sender<Message> {
        &self.0.message_sender
    }

    pub fn message_receiver(&self) -> broadcast::Receiver<Message> {
        self.0.message_sender.subscribe()
    }

    pub fn prohibit_mutation(&self) -> bool {
        self.0.prohibit_mutation.load(atomic::Ordering::SeqCst)
    }

    pub fn set_prohibit_mutation(&self, prohibit_mutation: bool) -> bool {
        self.0
            .prohibit_mutation
            .swap(prohibit_mutation, atomic::Ordering::SeqCst)
    }

    pub fn server_graphql_endpoints_read(&self) -> RwLockReadGuard<GraphQLEndpoints> {
        self.0.server_endpoints.read()
    }

    pub fn server_graphql_endpoints_write(&self) -> RwLockWriteGuard<GraphQLEndpoints> {
        self.0.server_endpoints.write()
    }

    pub fn request_headers(&self) -> &Arc<RwLock<HeaderMap>> {
        &self.0.request_headers
    }

    pub fn response_headers(&self) -> &Arc<RwLock<HeaderMap>> {
        &self.0.response_headers
    }
}
