#![forbid(unsafe_code)]
#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::unreachable)]

mod admin_state;
mod app_state;
mod cli;
mod cli_query;
mod endpoints;
mod error;
mod model;
mod utils;

use std::net::ToSocketAddrs;

use admin_state::AdminState;
use app_state::AppState;
use async_graphql::{SDLExportOptions, Schema};
use axum_helpers::app::AxumApp;
use clap::Parser;
use cli::{Cli, Command};
use cli_query::{execute_cli_query, subscribe_to_messages};
use endpoints::router::routes;
use error::{
    CannotParseBoolFromEnvVarError, UnspecifiedGraphQLEndpointError,
    UnspecifiedGraphQLWsEndpointError,
};
use http::HeaderMap;
use model::{mutation::Mutation, query::Query, subscription::Subscription};

fn create_admin_schema(admin_state: AdminState) -> Schema<Query, Mutation, Subscription> {
    let query = Query {
        admin_state: admin_state.clone(),
    };
    let mutation = Mutation {
        admin_state: admin_state.clone(),
    };
    let subscription: Subscription = Subscription {
        admin_state: admin_state.clone(),
    };
    Schema::build(query, mutation, subscription).finish()
}

async fn serve(
    schema: Schema<Query, Mutation, Subscription>,
    admin_state: AdminState,
    listener_address: impl AsRef<str>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("starting application in server mode");

    let app_state = AppState::new(admin_state)?;

    let mut app = AxumApp::new(routes(app_state, schema));

    for addr in listener_address.as_ref().to_socket_addrs()? {
        if let Err(e) = app.spawn_server(addr).await {
            log::error!(
                "{}, could not listen on address = {addr}, error = {e:?}",
                log_location!()
            );
        }
    }

    app.join().await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        // .filter_level(log::LevelFilter::Debug)
        .filter_module("axum_helpers", log::LevelFilter::Debug)
        .filter_module("graphql_proxy", log::LevelFilter::Trace)
        .filter_module("tower_http", log::LevelFilter::Debug)
        .init();

    let _ = dotenvy::dotenv_override()
        .inspect_err(|e| log::warn!("Could not load .env file, error = {e}"));

    let cli = Cli::parse();

    const DEFAULT_SERVER_GRAPHQL_ENDPOINT_ENV_VARNAME: &str = "DEFAULT_SERVER_GRAPHQL_ENDPOINT";
    const DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT_ENV_VARNAME: &str =
        "DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT";
    const DEFAULT_PROHIBIT_MUTATION_ENV_VARNAME: &str = "DEFAULT_PROHIBIT_MUTATION";

    match cli.command {
        Command::Serve(params) => {
            let admin_state = AdminState::new(
                params
                    .server_graphql_endpoint
                    .or_else(|| std::env::var(DEFAULT_SERVER_GRAPHQL_ENDPOINT_ENV_VARNAME).ok())
                    .ok_or(UnspecifiedGraphQLEndpointError)?,
                params
                    .server_graphql_ws_endpoint
                    .or_else(|| std::env::var(DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT_ENV_VARNAME).ok())
                    .ok_or(UnspecifiedGraphQLWsEndpointError)?,
                params
                    .prohibit_mutation
                    .then_some(Ok(true))
                    .or_else(|| {
                        std::env::var(DEFAULT_PROHIBIT_MUTATION_ENV_VARNAME)
                            .ok()
                            .map(|value| {
                                value
                                    .parse::<bool>()
                                    .map_err(|e| CannotParseBoolFromEnvVarError {
                                        varname: DEFAULT_PROHIBIT_MUTATION_ENV_VARNAME.to_string(),
                                        source: e,
                                    })
                            })
                    })
                    .transpose()?
                    .unwrap_or(false),
                params.request_headers.into_iter().collect(),
                params.response_headers.into_iter().collect(),
            );

            let schema = create_admin_schema(admin_state.clone());
            serve(schema, admin_state, params.listener_address).await?
        }
        Command::Sdl => {
            let schema = create_admin_schema(AdminState::new(
                "",
                "",
                false,
                HeaderMap::default(),
                HeaderMap::default(),
            ));
            println!(
                "{}",
                schema.sdl_with_options(SDLExportOptions::new().prefer_single_line_descriptions())
            );
        }
        Command::Query(params) => execute_cli_query(params).await?,
        Command::SubscribeToMessages(params) => subscribe_to_messages(params).await?,
    }

    Ok(())
}
