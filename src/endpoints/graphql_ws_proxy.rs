use async_graphql::{Response, ServerError};
use async_graphql_axum::GraphQLResponse;
use axum::{
    extract::{
        ws::{CloseFrame as AxumCloseFrame, Message as AxumWsMessage, WebSocket},
        WebSocketUpgrade,
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    tungstenite::{
        client::IntoClientRequest,
        protocol::{
            frame::coding::CloseCode as TungsteniteCloseCode, CloseFrame as TungsteniteCloseFrame,
        },
        Message as TungsteniteMessage,
    },
    MaybeTlsStream, WebSocketStream,
};

use crate::{app_state::AppState, log_location, utils::move_and_replace_headers};

pub async fn get_graphql_ws_proxy(
    mut headers: HeaderMap,
    axum::extract::State(state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, GraphQLResponse> {
    let server_endpoint = state
        .control_state()
        .server_graphql_endpoints_read()
        .graphql_ws_endpoint
        .clone();

    log::debug!(
        "Starting ws connection with endpoint: '{}'",
        server_endpoint
    );

    let mut request = server_endpoint
        .into_client_request()
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

    log::debug!("GaphQL WS request headers = {:?}", request.headers());

    const PROHIBITED_HEADER_NAMES_TO_SERVER: &[&str] = &[
        "host",
        "content-length",
        "connection",
        "upgrade",
        "sec-websocket-key",
        "sec-websocket-version",
    ];

    move_and_replace_headers(
        request.headers_mut(),
        &mut headers,
        PROHIBITED_HEADER_NAMES_TO_SERVER,
    );

    let (ws_stream, mut server_response) = tokio_tungstenite::connect_async(request)
        .await
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

    log::debug!("Websocket server response = {:?}", server_response);

    let mut response = ws
        .protocols(["graphql-transport-ws"])
        .on_upgrade(|socket| handle_socket(socket, ws_stream));

    const PROHIBITED_HEADER_NAMES_TO_CLIENT: &[&str] = &[
        "host",
        "content-length",
        "connection",
        "upgrade",
        "sec-websocket-key",
        "sec-websocket-extensions",
        "sec-websocket-version",
        "sec-websocket-protocol",
        "sec-websocket-accept",
    ];

    move_and_replace_headers(
        response.headers_mut(),
        server_response.headers_mut(),
        PROHIBITED_HEADER_NAMES_TO_CLIENT,
    );

    Ok(response)
}

async fn handle_socket(
    mut client_stream: WebSocket,
    mut server_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    let (server_to_client_sender, mut server_to_client_receiver) = mpsc::unbounded_channel();
    let (client_to_server_sender, mut client_to_server_receiver) = mpsc::unbounded_channel();

    // handling server stream
    tokio::spawn(async move {
        loop {
            tokio::select! {
                message = server_stream.next() => {
                    match message {
                        Some(Ok(message)) => {
                            if server_to_client_sender.send(message).is_err() {
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            log::error!("{}, error reading from server, error = '{e}'", log_location!());
                            break;
                        }
                        None => break,
                    }
                }
                message = client_to_server_receiver.recv() => {
                    match message {
                        Some(message) => {
                            if server_stream
                                .send(axum_to_tungstenite_message(message))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // handling client stream
    loop {
        tokio::select! {
            message = client_stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        if client_to_server_sender.send(message).is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        log::error!("{}, error reading from client, error = '{e}'", log_location!());
                    }
                    None => break,
                }
            }
            message = server_to_client_receiver.recv() => {
                match message {
                    Some(message) => {
                        if client_stream
                            .send(tungstenite_to_axum_message(message))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
}

fn tungstenite_to_axum_message(message: TungsteniteMessage) -> AxumWsMessage {
    match message {
        TungsteniteMessage::Binary(value) => AxumWsMessage::Binary(value),
        TungsteniteMessage::Close(value) => {
            AxumWsMessage::Close(value.map(tungstenite_to_axum_close_frame))
        }
        TungsteniteMessage::Frame(_value) => {
            log::error!(
                "{}, according to the docs of tungstenite, this should not happen",
                log_location!()
            );
            unreachable!();
        }
        TungsteniteMessage::Ping(value) => AxumWsMessage::Ping(value),
        TungsteniteMessage::Pong(value) => AxumWsMessage::Pong(value),
        TungsteniteMessage::Text(value) => AxumWsMessage::Text(value),
    }
}

fn axum_to_tungstenite_message(message: AxumWsMessage) -> TungsteniteMessage {
    match message {
        AxumWsMessage::Binary(value) => TungsteniteMessage::Binary(value),
        AxumWsMessage::Close(value) => {
            TungsteniteMessage::Close(value.map(axum_to_tungstenite_close_frame))
        }
        AxumWsMessage::Ping(value) => TungsteniteMessage::Ping(value),
        AxumWsMessage::Pong(value) => TungsteniteMessage::Pong(value),
        AxumWsMessage::Text(value) => TungsteniteMessage::Text(value),
    }
}

fn tungstenite_to_axum_close_frame(close_frame: TungsteniteCloseFrame) -> AxumCloseFrame {
    AxumCloseFrame {
        code: close_frame.code.into(),
        reason: close_frame.reason,
    }
}

fn axum_to_tungstenite_close_frame(close_frame: AxumCloseFrame) -> TungsteniteCloseFrame {
    TungsteniteCloseFrame {
        code: TungsteniteCloseCode::from(close_frame.code),
        reason: close_frame.reason,
    }
}
