use clap::Parser;

#[derive(Debug, Parser)]
pub struct ServeParams {
    #[arg(
        short('l'),
        long("listener-address"),
        help("Address where the proxy server accepts the connections (e.g., 127.0.0.1:8000)")
    )]
    pub listener_address: String,
}

#[derive(Debug, Parser)]
pub enum Commands {
    Serve(ServeParams),
    Sdl,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

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
}
