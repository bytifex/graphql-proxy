use async_graphql::Object;

use crate::control_state::ControlState;

use super::types::graphql_endpoints::GraphQLEndpoints;

pub struct Mutation {
    pub control_state: ControlState,
}

#[Object]
impl Mutation {
    pub async fn set_server_endpoints(
        &self,
        #[graphql(name = "graphQlEndpoint")] graphql_endpoint: String,
        #[graphql(name = "graphQlWsEndpoint")] graphql_ws_endpoint: String,
    ) -> GraphQLEndpoints {
        let mut endpoints = GraphQLEndpoints {
            graphql_endpoint,
            graphql_ws_endpoint,
        };

        std::mem::swap(
            &mut endpoints,
            &mut self.control_state.server_graphql_endpoints_write(),
        );

        endpoints
    }
}
