use std::sync::Arc;

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::model::types::graphql_endpoints::GraphQLEndpoints;

struct ControlStateInner {
    prohibit_mutation: bool,
    server_endpoints: RwLock<GraphQLEndpoints>,
}

#[derive(Clone)]
pub struct ControlState(Arc<ControlStateInner>);

impl ControlState {
    pub fn new(
        server_graphql_endpoint: impl Into<String>,
        server_graphql_ws_endpoint: impl Into<String>,
    ) -> Self {
        Self(Arc::new(ControlStateInner {
            prohibit_mutation: true,
            server_endpoints: RwLock::new(GraphQLEndpoints {
                graphql_endpoint: server_graphql_endpoint.into(),
                graphql_ws_endpoint: server_graphql_ws_endpoint.into(),
            }),
        }))
    }

    pub fn prohibit_mutation(&self) -> bool {
        self.0.prohibit_mutation
    }

    pub fn server_graphql_endpoints_read(&self) -> RwLockReadGuard<GraphQLEndpoints> {
        self.0.server_endpoints.read()
    }

    pub fn server_graphql_endpoints_write(&self) -> RwLockWriteGuard<GraphQLEndpoints> {
        self.0.server_endpoints.write()
    }
}
