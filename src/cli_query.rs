use async_graphql::InputType;
use graphql_cli_tools::client::{execute, load_variables, ws_request, GraphQlResponse};

use crate::cli::{QueryParams, SubscribeMessagesParams};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionMessage {
    message: String,
    message_counter: Option<u64>,
    message_type: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionResult {
    #[serde(rename = "messages")]
    message: MessageSubscriptionMessage,
}

fn response_processor(response: GraphQlResponse) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    Ok(())
}

pub async fn execute_cli_query(params: QueryParams) -> Result<(), Box<dyn std::error::Error>> {
    let headers = params.headers.into_iter().collect();

    execute(
        params.server_endpoint,
        headers,
        params.query_path,
        params.operation_name,
        load_variables(params.variables_from_json, params.variables)?,
        response_processor,
        params
            .try_reconnect_duration
            .map(|duration| duration.into()),
    )
    .await
}

pub async fn subscribe_to_messages(
    params: SubscribeMessagesParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let headers = params.headers.into_iter().collect();

    let mut variables = serde_json::Map::<String, serde_json::Value>::new();
    variables.insert(
        "messageFilters".to_string(),
        serde_json::Value::Array(
            params
                .message_filters
                .into_iter()
                .map(|filter| filter.to_value().into_json())
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );

    ws_request(
        params.server_endpoint,
        headers,
        include_str!("graphql_queries/subscribe-to-messages.graphql").to_string(),
        Option::<&str>::None,
        variables,
        response_processor,
        params
            .try_reconnect_duration
            .map(|duration| duration.into()),
    )
    .await
}
