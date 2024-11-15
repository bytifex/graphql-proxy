use async_graphql::Object;

use crate::admin_state::AdminState;

use super::{
    scalars::{header_name_scalar::HeaderNameScalar, header_value_scalar::HeaderValueScalar},
    types::{graphql_endpoints::GraphQLEndpoints, header::Header},
};

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

    pub async fn request_header(&self, name: HeaderNameScalar) -> Option<HeaderValueScalar> {
        self.admin_state
            .request_headers_read()
            .get(name.as_header_name())
            .cloned()
            .map(|item| item.into())
    }

    pub async fn response_header(&self, name: HeaderNameScalar) -> Option<HeaderValueScalar> {
        self.admin_state
            .response_headers_read()
            .get(name.as_header_name())
            .cloned()
            .map(|item| item.into())
    }

    pub async fn request_headers(&self) -> Vec<Header> {
        self.admin_state
            .request_headers_read()
            .iter()
            .map(|(name, value)| Header {
                name: name.clone().into(),
                value: value.clone().into(),
            })
            .collect()
    }

    pub async fn response_headers(&self) -> Vec<Header> {
        self.admin_state
            .response_headers_read()
            .iter()
            .map(|(name, value)| Header {
                name: name.clone().into(),
                value: value.clone().into(),
            })
            .collect()
    }
}
