use async_graphql::Object;

use crate::admin_state::AdminState;

use super::types::{graphql_endpoints::GraphQLEndpoints, headers::Headers};

pub struct Query {
    pub admin_state: AdminState,
}

#[Object]
impl Query {
    pub async fn prohibit_mutation(&self) -> bool {
        self.admin_state.prohibit_mutation()
    }

    pub async fn server_endpoints(&self) -> GraphQLEndpoints {
        self.admin_state.server_graphql_endpoints_read().clone()
    }

    pub async fn request_headers(&self) -> Headers {
        Headers::from_rw_lock_header_map(self.admin_state.request_headers().clone())
    }

    pub async fn response_headers(&self) -> Headers {
        Headers::from_rw_lock_header_map(self.admin_state.response_headers().clone())
    }
}
