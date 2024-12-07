use std::sync::{
    atomic::{self, AtomicU64},
    Arc,
};

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
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
};
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

use crate::{
    admin_state::ConnectionId,
    app_state::AppState,
    log_location,
    model::{
        enums::{connection_type::ConnectionType, message_direction::MessageDirection},
        types::{headers::Headers, message::Message},
    },
    utils::move_and_replace_headers,
};

pub async fn get_graphql_ws_proxy(
    mut headers: HeaderMap,
    axum::extract::State(state): axum::extract::State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, GraphQLResponse> {
    let server_endpoint_url = Arc::new(
        state
            .admin_state()
            .server_graphql_endpoints_read()
            .graphql_ws_endpoint
            .clone(),
    );

    log::debug!(
        "Starting ws connection with endpoint: '{}'",
        server_endpoint_url
    );

    log::debug!("GaphQL WS request headers = {:?}", headers);

    let mut request = server_endpoint_url
        .as_ref()
        .into_client_request()
        .inspect_err(|e| log::error!("{}, {}", log_location!(), e.to_string()))
        .map_err(|e| {
            GraphQLResponse::from(Response::from_errors(vec![ServerError::new(
                e.to_string(),
                None,
            )]))
        })?;

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

    let mut additional_request_headers = state.admin_state().request_headers().read().clone();
    move_and_replace_headers(request.headers_mut(), &mut additional_request_headers, &[]);

    let message_sender = state.admin_state().message_sender_ref().clone();

    let connection_id = ConnectionId::new();
    let sequence_counter = Arc::new(AtomicU64::new(0));
    send_message_to_subscriptions(
        connection_id.clone(),
        &sequence_counter,
        serde_json::Value::Null,
        MessageDirection::Request,
        &message_sender,
        Some(Arc::new(Headers::from_header_map(
            request.headers().clone(),
        ))),
        server_endpoint_url.clone(),
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

    let mut response = {
        let message_sender = message_sender.clone();
        let connection_id = connection_id.clone();
        let sequence_counter = sequence_counter.clone();
        let server_endpoint_url = server_endpoint_url.clone();

        ws.on_upgrade(move |socket| {
            handle_socket(
                socket,
                ws_stream,
                message_sender,
                connection_id,
                sequence_counter,
                server_endpoint_url,
            )
        })
    };

    const PROHIBITED_HEADER_NAMES_TO_CLIENT: &[&str] = &[
        "host",
        "content-length",
        "connection",
        "upgrade",
        "sec-websocket-key",
        "sec-websocket-extensions",
        "sec-websocket-version",
        "sec-websocket-accept",
    ];

    move_and_replace_headers(
        response.headers_mut(),
        server_response.headers_mut(),
        PROHIBITED_HEADER_NAMES_TO_CLIENT,
    );

    let mut additional_response_headers = state.admin_state().response_headers().read().clone();
    move_and_replace_headers(
        response.headers_mut(),
        &mut additional_response_headers,
        &[],
    );

    send_message_to_subscriptions(
        connection_id.clone(),
        &sequence_counter,
        serde_json::Value::Null,
        MessageDirection::Response,
        &message_sender,
        Some(Arc::new(Headers::from_header_map(
            response.headers().clone(),
        ))),
        server_endpoint_url,
    );

    Ok(response)
}

async fn handle_socket(
    client_stream: WebSocket,
    server_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    message_sender: broadcast::Sender<Message>,
    connection_id: ConnectionId,
    sequence_counter: Arc<AtomicU64>,
    server_endpoint_url: Arc<String>,
) {
    let (server_to_client_sender, server_to_client_receiver) = mpsc::unbounded_channel();
    let (client_to_server_sender, client_to_server_receiver) = mpsc::unbounded_channel();

    {
        let message_sender = message_sender.clone();
        let connection_id = connection_id.clone();
        let sequence_counter = sequence_counter.clone();
        let server_endpoint_url = server_endpoint_url.clone();

        tokio::spawn(async move {
            handle_server_stream(
                connection_id,
                sequence_counter,
                server_stream,
                server_to_client_sender,
                client_to_server_receiver,
                message_sender,
                server_endpoint_url,
            )
            .await;
        });
    }

    handle_client_stream(
        connection_id,
        sequence_counter,
        client_stream,
        client_to_server_sender,
        server_to_client_receiver,
        message_sender,
        server_endpoint_url,
    )
    .await;
}

async fn handle_server_stream(
    connection_id: ConnectionId,
    sequence_counter: Arc<AtomicU64>,
    mut server_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    server_to_client_sender: mpsc::UnboundedSender<AxumWsMessage>,
    mut client_to_server_receiver: mpsc::UnboundedReceiver<AxumWsMessage>,
    message_sender: broadcast::Sender<Message>,
    server_endpoint_url: Arc<String>,
) {
    loop {
        tokio::select! {
            message = server_stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        let message = tungstenite_to_axum_message(message);
                        send_axum_ws_message_to_subscriptions(
                            connection_id.clone(),
                            &sequence_counter,
                            &message,
                            MessageDirection::Response,
                            &message_sender,
                            None,
                            server_endpoint_url.clone(),
                        );

                        if server_to_client_sender.send(message).is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        log::error!("{}, error reading from server, error = '{e}'", log_location!());
                    }
                    None => {
                        log::debug!("connection to the server is closed");
                        break;
                    }
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
}

async fn handle_client_stream(
    connection_id: ConnectionId,
    sequence_counter: Arc<AtomicU64>,
    mut client_stream: WebSocket,
    client_to_server_sender: mpsc::UnboundedSender<AxumWsMessage>,
    mut server_to_client_receiver: mpsc::UnboundedReceiver<AxumWsMessage>,
    message_sender: broadcast::Sender<Message>,
    server_endpoint_url: Arc<String>,
) {
    loop {
        tokio::select! {
            message = client_stream.next() => {
                match message {
                    Some(Ok(message)) => {
                        send_axum_ws_message_to_subscriptions(
                            connection_id.clone(),
                            &sequence_counter,
                            &message,
                            MessageDirection::Request,
                            &message_sender,
                            None,
                            server_endpoint_url.clone(),
                        );
                        if client_to_server_sender.send(message).is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        log::error!("{}, error reading from client, error = '{e}'", log_location!());
                        if client_to_server_sender.send(AxumWsMessage::Text(e.to_string())).is_err() {
                            break;
                        }
                    }
                    None => {
                        log::debug!("connection to the client is closed");
                        break;
                    }
                }
            }
            message = server_to_client_receiver.recv() => {
                match message {
                    Some(message) => {
                        if client_stream
                            .send(message)
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

fn send_message_to_subscriptions(
    connection_id: ConnectionId,
    sequence_counter: &AtomicU64,
    message: serde_json::Value,
    message_direction: MessageDirection,
    message_sender: &broadcast::Sender<Message>,
    transmitted_headers: Option<Arc<Headers>>,
    server_endpoint_url: Arc<String>,
) {
    let sequence_counter = sequence_counter.fetch_add(1, atomic::Ordering::SeqCst);
    let _ = message_sender.send(Message {
        connection_id: connection_id.as_arc_string(),
        message: Arc::new(message),
        sequence_counter,
        connection_type: ConnectionType::Ws,
        message_direction,
        transmitted_headers,
        server_endpoint_url,
    });
}

fn send_axum_ws_message_to_subscriptions(
    connection_id: ConnectionId,
    sequence_counter: &AtomicU64,
    message: &AxumWsMessage,
    message_direction: MessageDirection,
    message_sender: &broadcast::Sender<Message>,
    transmitted_headers: Option<Arc<Headers>>,
    server_endpoint_url: Arc<String>,
) {
    if message_sender.receiver_count() != 0 {
        match message {
            AxumWsMessage::Text(text) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                    send_message_to_subscriptions(
                        connection_id,
                        sequence_counter,
                        json,
                        message_direction,
                        message_sender,
                        transmitted_headers,
                        server_endpoint_url,
                    );
                } else {
                    send_message_to_subscriptions(
                        connection_id,
                        sequence_counter,
                        serde_json::Value::from(text.clone()),
                        message_direction,
                        message_sender,
                        transmitted_headers,
                        server_endpoint_url,
                    );
                }
            }
            AxumWsMessage::Binary(value) => {
                send_message_to_subscriptions(
                    connection_id,
                    sequence_counter,
                    serde_json::Value::from(value.clone()),
                    message_direction,
                    message_sender,
                    transmitted_headers,
                    server_endpoint_url,
                );
            }
            _ => (),
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
