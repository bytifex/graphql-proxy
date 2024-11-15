use std::convert::Infallible;

use async_graphql::Schema;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    routing::{get, post, post_service},
    Router,
};
use axum_helpers::response_http_header_mutator::ResponseHttpHeaderMutatorLayer;
use config::{
    ADMIN_GRAPHQL_ENDPOINT, ADMIN_GRAPHQL_WS_ENDPOINT, PROXIED_GRAPHQL_ENDPOINT,
    PROXIED_GRAPHQL_WS_ENDPOINT,
};
use tower_http::trace::TraceLayer;

use crate::{
    model::{mutation::Mutation, query::Query, subscription::Subscription},
    AppState,
};

use super::{
    get_index, graphiql, graphql_proxy::post_graphql_proxy, graphql_ws_proxy::get_graphql_ws_proxy,
    options_graphql,
};

mod config {
    pub const ADMIN_GRAPHQL_ENDPOINT: &str = "/admin-api/graphql";
    pub const ADMIN_GRAPHQL_WS_ENDPOINT: &str = "/admin-api/graphql-ws";

    pub const PROXIED_GRAPHQL_ENDPOINT: &str = "/api/graphql";
    pub const PROXIED_GRAPHQL_WS_ENDPOINT: &str = "/api/graphql-ws";
}

pub fn routes(state: AppState, schema: Schema<Query, Mutation, Subscription>) -> Router {
    let preflight_middleware = ResponseHttpHeaderMutatorLayer::new(|_req_headers, res_headers| {
        res_headers.insert(
            "Access-Control-Allow-Methods",
            "*".parse().expect("cannot parse HTTP header value"),
        );
        res_headers.insert(
            "Access-Control-Allow-Headers",
            "*".parse().expect("cannot parse HTTP header value"),
        );
        res_headers.insert(
            "Access-Control-Allow-Origin",
            "*".parse().expect("cannot parse HTTP header value"),
        );

        Ok::<(), Infallible>(())
    });

    let admin_graphql_routes = Router::new()
        .route(
            "/admin-graphiql",
            get(|| graphiql(ADMIN_GRAPHQL_ENDPOINT, ADMIN_GRAPHQL_WS_ENDPOINT)),
        )
        .route_service(
            ADMIN_GRAPHQL_WS_ENDPOINT,
            GraphQLSubscription::new(schema.clone()),
        )
        .route(
            ADMIN_GRAPHQL_ENDPOINT,
            post_service(GraphQL::new(schema))
                .options(options_graphql)
                .route_layer(preflight_middleware.clone()),
        );

    let proxied_graphql_route = Router::new()
        .route(
            "/graphiql",
            get(|| graphiql(PROXIED_GRAPHQL_ENDPOINT, PROXIED_GRAPHQL_WS_ENDPOINT)),
        )
        .route(PROXIED_GRAPHQL_WS_ENDPOINT, get(get_graphql_ws_proxy))
        .route(
            PROXIED_GRAPHQL_ENDPOINT,
            post(post_graphql_proxy)
                .options(options_graphql)
                .route_layer(preflight_middleware.clone()),
        );

    Router::new()
        .route("/", get(get_index))
        .merge(admin_graphql_routes)
        .merge(proxied_graphql_route)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
