use async_graphql::InputType;
use graphql_cli_tools::client::{execute, load_variables, ws_request, GraphQlResponse};

use crate::{
    cli::{QueryParams, SubscribeMessagesParams},
    model::enums::{connection_type::ConnectionType, message_direction::MessageDirection},
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionHeader {
    name: String,
    value: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionHeaders {
    all: Vec<MessageSubscriptionHeader>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionMessage {
    message: serde_json::Value,
    connection_id: String,
    sequence_counter: usize,
    connection_type: ConnectionType,
    message_direction: MessageDirection,
    transmitted_headers: Option<MessageSubscriptionHeaders>,
    server_endpoint_url: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSubscriptionResult {
    #[serde(rename = "messages")]
    message: MessageSubscriptionMessage,
}

fn response_processor(
    response: GraphQlResponse,
    print_curl_command: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let curl_command = if print_curl_command {
        if let Some(data) = &response.data {
            if let Ok(result) = serde_json::from_value::<MessageSubscriptionResult>(data.clone()) {
                if let (
                    Some(transmitted_headers),
                    MessageDirection::Request,
                    ConnectionType::Http,
                ) = (
                    &result.message.transmitted_headers,
                    result.message.message_direction,
                    result.message.connection_type,
                ) {
                    let mut curl_command = format!(
                        "curl -X POST '{}' -H 'Content-Type: application/json'",
                        result.message.server_endpoint_url,
                    );

                    for header in transmitted_headers.all.iter() {
                        curl_command.push_str(&format!(" -H '{}: {}'", header.name, header.value));
                    }

                    curl_command.push_str(&format!(" -d '{}'", result.message.message));

                    Some(curl_command)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    println!("{}", serde_json::to_string_pretty(&response)?);
    if let Some(curl_command) = curl_command {
        println!("CurlCommand = `{}`", curl_command);
    }
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
        |response| response_processor(response, false),
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
    variables.insert(
        "includeTransmittedHeaders".to_string(),
        serde_json::Value::Bool(params.transmitted_headers),
    );

    ws_request(
        params.server_endpoint,
        headers,
        include_str!("graphql_queries/subscribe-to-messages.graphql").to_string(),
        Option::<&str>::None,
        variables,
        |response| response_processor(response, params.as_curl_command),
        params
            .try_reconnect_duration
            .map(|duration| duration.into()),
    )
    .await
}
