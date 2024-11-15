use async_graphql::Object;

use crate::admin_state::AdminState;

use super::{
    scalars::{header_name_scalar::HeaderNameScalar, header_value_scalar::HeaderValueScalar},
    types::graphql_endpoints::GraphQLEndpoints,
};

pub struct Mutation {
    pub admin_state: AdminState,
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
            &mut self.admin_state.server_graphql_endpoints_write(),
        );

        endpoints
    }

    pub async fn set_prohibit_mutation(&self, prohibit_mutation: bool) -> bool {
        self.admin_state.set_prohibit_mutation(prohibit_mutation)
    }

    pub async fn add_request_header(
        &self,
        name: HeaderNameScalar,
        value: HeaderValueScalar,
    ) -> bool {
        let mut headers = self.admin_state.request_headers_write();

        headers.insert(name.as_header_name().clone(), value.0);

        true
    }

    pub async fn set_request_header(
        &self,
        name: HeaderNameScalar,
        value: HeaderValueScalar,
    ) -> Option<HeaderValueScalar> {
        let mut headers = self.admin_state.request_headers_write();

        let old_header_value = headers.remove(name.as_header_name());
        headers.insert(name.as_header_name().clone(), value.0);

        old_header_value.map(|item| item.into())
    }

    pub async fn remove_request_header(&self, name: HeaderNameScalar) -> Option<HeaderValueScalar> {
        self.admin_state
            .request_headers_write()
            .remove(name.as_header_name())
            .map(|item| item.into())
    }

    pub async fn add_response_header(
        &self,
        name: HeaderNameScalar,
        value: HeaderValueScalar,
    ) -> bool {
        let mut headers = self.admin_state.response_headers_write();

        headers.insert(name.as_header_name().clone(), value.0);

        true
    }

    pub async fn set_response_header(
        &self,
        name: HeaderNameScalar,
        value: HeaderValueScalar,
    ) -> Option<HeaderValueScalar> {
        let mut headers = self.admin_state.response_headers_write();

        let old_header_value = headers.remove(name.as_header_name());
        headers.insert(name.as_header_name().clone(), value.0);

        old_header_value.map(|item| item.into())
    }

    pub async fn remove_response_header(
        &self,
        name: HeaderNameScalar,
    ) -> Option<HeaderValueScalar> {
        self.admin_state
            .response_headers_write()
            .remove(name.as_header_name())
            .map(|item| item.into())
    }
}
