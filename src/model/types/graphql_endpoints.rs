use async_graphql::Object;

#[derive(Debug, Clone)]
pub struct GraphQLEndpoints {
    pub graphql_endpoint: String,
    pub graphql_ws_endpoint: String,
}

#[Object(name = "GraphQlEndpoints")]
impl GraphQLEndpoints {
    #[graphql(name = "graphQlEndpoint")]
    async fn graphql_endpoint(&self) -> &String {
        &self.graphql_endpoint
    }

    #[graphql(name = "graphQlWsEndpoint")]
    async fn graphql_ws_endpoint(&self) -> &String {
        &self.graphql_ws_endpoint
    }
}
