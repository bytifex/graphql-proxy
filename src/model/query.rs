use async_graphql::Object;

use crate::control_state::ControlState;

use super::types::graphql_endpoints::GraphQLEndpoints;

pub struct Query {
    pub control_state: ControlState,
}

#[Object]
impl Query {
    pub async fn prohibit_mutation(&self) -> bool {
        self.control_state.prohibit_mutation()
    }

    pub async fn server_endpoints(&self) -> GraphQLEndpoints {
        self.control_state.server_graphql_endpoints_read().clone()
    }
}
