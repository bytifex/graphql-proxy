use std::sync::Arc;

use async_graphql::{Response, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use async_graphql_parser::{
    parse_query,
    types::{DocumentOperations, ExecutableDocument, OperationType},
};
use axum::{body::Body, http::HeaderMap, response::IntoResponse};
use tokio::sync::broadcast;

use crate::{
    admin_state::ConnectionId,
    app_state::AppState,
    log_location,
    model::{
        enums::{connection_type::ConnectionType, message_direction::MessageDirection},
        types::message::Message,
    },
    utils::move_and_replace_headers,
};

pub async fn post_graphql_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    graphql_request: GraphQLRequest,
) -> Result<impl IntoResponse, GraphQLResponse> {
    log::debug!("GaphQL request headers = {:?}", headers);

    let connection_id = ConnectionId::new();
    let mut sequence_counter = 0;

    let message_sender = state.admin_state().message_sender_ref().clone();
    if message_sender.receiver_count() != 0 {
        let _ = message_sender.send(Message {
            connection_id: connection_id.as_arc_string(),
            message: Arc::new(serde_json::json!(graphql_request.0)),
            sequence_counter,
            connection_type: ConnectionType::Http,
            message_direction: MessageDirection::Request,
        });
    }
    sequence_counter += 1;

    let parsed_graphql_query = parse_query(&graphql_request.0.query)
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

    if state.admin_state().prohibit_mutation()
        && is_query_of_type(
            graphql_request.0.operation_name.as_ref(),
            parsed_graphql_query,
            OperationType::Mutation,
        )
    {
        return Err(GraphQLResponse::from(Response::from_errors(vec![
            ServerError::new("Mutations are set to be prohibited", None),
        ])));
    }

    const PROHIBITED_HEADER_NAMES_TO_SERVER: &[&str] = &["host", "content-length", "content-type"];

    let mut request_headers = headers
        .iter()
        .filter_map(|(name, value)| {
            if PROHIBITED_HEADER_NAMES_TO_SERVER.contains(&name.as_str()) {
                None
            } else {
                Some((name.clone(), value.clone()))
            }
        })
        .collect();

    let mut additional_request_headers = state.admin_state().request_headers_read().clone();
    move_and_replace_headers(&mut request_headers, &mut additional_request_headers, &[]);

    let endpoints = state.admin_state().server_graphql_endpoints_read().clone();

    let server_response = state
        .server_client()
        .post(endpoints.graphql_endpoint)
        .headers(request_headers)
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

    let additional_response_headers = state.admin_state().response_headers_read().clone();
    process_server_response(
        connection_id,
        sequence_counter,
        message_sender,
        server_response,
        additional_response_headers,
    )
    .await
}

fn is_query_of_type(
    operation_name: Option<impl AsRef<str>>,
    parsed_graphql_query: ExecutableDocument,
    query_type: OperationType,
) -> bool {
    match &parsed_graphql_query.operations {
        DocumentOperations::Single(operation) => operation.node.ty == query_type,
        DocumentOperations::Multiple(operations) => {
            if let Some(operation_name) = operation_name {
                if let Some(operation) = operations.get(operation_name.as_ref()) {
                    operation.node.ty == query_type
                } else {
                    false
                }
            } else {
                operations
                    .iter()
                    .any(|(_name, operation)| operation.node.ty == query_type)
            }
        }
    }
}

async fn process_server_response(
    connection_id: ConnectionId,
    sequence_counter: u64,
    message_sender: broadcast::Sender<Message>,
    mut server_response: reqwest::Response,
    mut additional_response_headers: HeaderMap,
) -> Result<impl IntoResponse, GraphQLResponse> {
    const PROHIBITED_HEADER_NAMES_TO_CLIENT: &[&str] = &[];

    let mut headers = HeaderMap::new();
    move_and_replace_headers(
        &mut headers,
        server_response.headers_mut(),
        PROHIBITED_HEADER_NAMES_TO_CLIENT,
    );

    move_and_replace_headers(&mut headers, &mut additional_response_headers, &[]);

    let text = server_response.text().await.map_err(|e| {
        log::error!("{}, {}", log_location!(), e.to_string());
        GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
            e.to_string(),
            None,
        )]))
    })?;

    if message_sender.receiver_count() != 0 {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            let _ = message_sender.send(Message {
                connection_id: connection_id.as_arc_string(),
                message: Arc::new(json),
                sequence_counter,
                connection_type: ConnectionType::Http,
                message_direction: MessageDirection::Response,
            });
        } else {
            let _ = message_sender.send(Message {
                connection_id: connection_id.as_arc_string(),
                message: Arc::new(serde_json::Value::from(text.clone())),
                sequence_counter,
                connection_type: ConnectionType::Http,
                message_direction: MessageDirection::Response,
            });
        }
    }

    Ok((headers, Body::from(text)))
}
