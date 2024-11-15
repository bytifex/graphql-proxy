use async_graphql::{Response, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use async_graphql_parser::{parse_query, types::OperationType};
use axum::{body::Body, http::HeaderMap, response::IntoResponse};

use crate::{app_state::AppState, log_location, utils::move_and_replace_headers};

pub async fn post_graphql_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    graphql_request: GraphQLRequest,
) -> Result<impl IntoResponse, GraphQLResponse> {
    log::debug!("GraphQL request = {:?}", graphql_request.0);

    log::debug!("GaphQL request headers = {:?}", headers);

    let parsed_graphql_query = parse_query(&graphql_request.0.query)
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

    let contains_mutation = parsed_graphql_query
        .operations
        .iter()
        .any(|(_name, operation)| operation.node.ty == OperationType::Mutation);
    if state.control_state().prohibit_mutation() && contains_mutation {
        return Err(GraphQLResponse::from(Response::from_errors(vec![
            ServerError::new("Mutations are set to be prohibited", None),
        ])));
    }

    const PROHIBITED_HEADER_NAMES_TO_SERVER: &[&str] = &["host", "content-length", "content-type"];

    let endpoints = state
        .control_state()
        .server_graphql_endpoints_read()
        .clone();

    let mut server_response = state
        .server_client()
        .post(endpoints.graphql_endpoint)
        .headers(
            headers
                .iter()
                .filter_map(|(name, value)| {
                    if PROHIBITED_HEADER_NAMES_TO_SERVER.contains(&name.as_str()) {
                        None
                    } else {
                        Some((name.clone(), value.clone()))
                    }
                })
                .collect(),
        )
        .json(&graphql_request.0)
        .send()
        .await
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

    log::debug!("Server response = {:?}", server_response);

    const PROHIBITED_HEADER_NAMES_TO_CLIENT: &[&str] = &[];

    let mut headers = HeaderMap::new();
    move_and_replace_headers(
        &mut headers,
        server_response.headers_mut(),
        PROHIBITED_HEADER_NAMES_TO_CLIENT,
    );

    Ok((headers, Body::from_stream(server_response.bytes_stream())))
}
