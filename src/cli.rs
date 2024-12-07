use std::path::PathBuf;

use clap::Parser;
use graphql_cli_tools::clap_types::{ClapHttpHeaderParser, ClapKeyJsonValueParser};
use http::{HeaderName, HeaderValue};

use crate::model::inputs::message_filter::{MessageFilter, MessageFilterCliParser};

#[derive(Debug, Parser)]
pub struct QueryParams {
    #[arg(
        short('e'),
        long("server-endpoint"),
        help("Endpoint where the server accepts the connections (e.g., http://localhost:8000/api/graphql)")
    )]
    pub server_endpoint: String,

    #[arg(
        short('q'),
        long("query-path"),
        help("Path of the query that has to be executed")
    )]
    pub query_path: PathBuf,

    #[arg(
        short('o'),
        long("operation-name"),
        help("Name of the operation that has to be executed")
    )]
    pub operation_name: Option<String>,

    #[arg(
        long("variables-from-json"),
        help("Json file containing variables to be sent to the server")
    )]
    pub variables_from_json: Option<PathBuf>,

    #[arg(
        short('v'),
        long("variable"),
        value_parser(ClapKeyJsonValueParser),
        help("Variable to be sent to the server")
    )]
    pub variables: Vec<(String, serde_json::Value)>,

    #[arg(
        long("http-header"),
        value_parser(ClapHttpHeaderParser),
        help("HTTP header to be sent to the server")
    )]
    pub headers: Vec<(HeaderName, HeaderValue)>,

    #[arg(
        short('r'),
        long("try-reconnect-duration"),
        help("When in subscription mode, the client will try to reconnect to the server if there is no connection (e.g., 500ms"),
    )]
    pub try_reconnect_duration: Option<humantime::Duration>,
}

#[derive(Debug, Parser)]
pub struct SubscribeMessagesParams {
    #[arg(
        short('e'),
        long("server-endpoint"),
        help("Endpoint where the server accepts the connections (e.g., http://localhost:8000/api/graphql)")
    )]
    pub server_endpoint: String,

    #[arg(
        long("http-header"),
        value_parser(ClapHttpHeaderParser),
        help("HTTP header to be sent to the server")
    )]
    pub headers: Vec<(HeaderName, HeaderValue)>,

    #[arg(
        value_enum,
        short('f'),
        long("message-filter"),
        value_parser(MessageFilterCliParser),
        help("Define a filter for the messages to be received")
    )]
    pub message_filters: Vec<MessageFilter>,

    #[arg(
        short('r'),
        long("try-reconnect-duration"),
        help("When in subscription mode, the client will try to reconnect to the server if there is no connection (e.g., 500ms"),
    )]
    pub try_reconnect_duration: Option<humantime::Duration>,

    #[arg(
        long("transmitted-headers"),
        help(
            "When set, the messages will contain the headers sent to the peer (proxied server or client)"
        )
    )]
    pub transmitted_headers: bool,

    #[arg(
        long("as-curl-command"),
        requires("transmitted_headers"),
        help("When set, the requests sent to the proxied server will be printed as curl commands (be aware that this flag sets transmitted-headers also)")
    )]
    pub as_curl_command: bool,
}

#[derive(Debug, Parser)]
pub struct ServeParams {
    #[arg(
        short('l'),
        long("listener-address"),
        help("Address where the proxy server accepts the connections (e.g., 127.0.0.1:8000)")
    )]
    pub listener_address: String,

    #[arg(
        short('s'),
        long("server-endpoint"),
        help(
            "Address where the server accepts the connections (e.g., https://someserver/api/graphql)"
        )
    )]
    pub server_graphql_endpoint: Option<String>,

    #[arg(
        short('w'),
        long("server-ws-endpoint"),
        help(
            "Address where the server accepts the connections (e.g., ws://someserver/api/graphql-ws)"
        )
    )]
    pub server_graphql_ws_endpoint: Option<String>,

    #[arg(
        short('m'),
        long("prohibit-mutation"),
        default_value("false"),
        help("Sets whether mutaitons proxied to the server are prohibited (default: false)")
    )]
    pub prohibit_mutation: bool,

    #[arg(
        long("response-header"),
        value_parser(ClapHttpHeaderParser),
        help("HTTP header to be sent to the server")
    )]
    pub response_headers: Vec<(HeaderName, HeaderValue)>,

    #[arg(
        long("request-header"),
        value_parser(ClapHttpHeaderParser),
        help("HTTP header to be sent to the client")
    )]
    pub request_headers: Vec<(HeaderName, HeaderValue)>,
}

#[derive(Debug, Parser)]
pub enum Command {
    Query(QueryParams),
    Serve(ServeParams),
    SubscribeToMessages(SubscribeMessagesParams),
    Sdl,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
