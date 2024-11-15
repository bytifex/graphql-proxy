#![allow(unreachable_code)]
#![allow(dead_code)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::unreachable)]

mod app_state;
mod cli;
mod control_state;
mod endpoints;
mod error;
mod model;
mod utils;

use std::net::ToSocketAddrs;

use app_state::AppState;
use async_graphql::{SDLExportOptions, Schema};
use axum_helpers::app::AxumApp;
use clap::Parser;
use cli::{Cli, Commands, ServeParams};
use control_state::ControlState;
use endpoints::router::routes;
use error::{UnspecifiedGraphQLEndpointError, UnspecifiedGraphQLWsEndpointError};
use model::{mutation::Mutation, query::Query, subscription::Subscription};

fn create_schema(control_state: ControlState) -> Schema<Query, Mutation, Subscription> {
    let query = Query {
        control_state: control_state.clone(),
    };
    let mutation = Mutation {
        control_state: control_state.clone(),
    };
    let subscription: Subscription = Subscription {
        _control_state: control_state.clone(),
    };
    Schema::build(query, mutation, subscription).finish()
}

async fn serve(
    schema: Schema<Query, Mutation, Subscription>,
    control_state: ControlState,
    ServeParams { listener_address }: ServeParams,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("starting application in server mode");

    let app_state = AppState::new(control_state)?;

    let mut app = AxumApp::new(routes(app_state, schema));

    for addr in listener_address.to_socket_addrs()? {
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

    let Cli {
        command,
        server_graphql_endpoint,
        server_graphql_ws_endpoint,
    } = Cli::parse();

    const DEFAULT_SERVER_GRAPHQL_ENDPOINT_ENV_VARNAME: &str = "DEFAULT_SERVER_GRAPHQL_ENDPOINT";
    const DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT_ENV_VARNAME: &str =
        "DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT";

    let control_state = ControlState::new(
        server_graphql_endpoint
            .or_else(|| std::env::var(DEFAULT_SERVER_GRAPHQL_ENDPOINT_ENV_VARNAME).ok())
            .ok_or(UnspecifiedGraphQLEndpointError)?,
        server_graphql_ws_endpoint
            .or_else(|| std::env::var(DEFAULT_SERVER_GRAPHQL_WS_ENDPOINT_ENV_VARNAME).ok())
            .ok_or(UnspecifiedGraphQLWsEndpointError)?,
    );
    let schema = create_schema(control_state.clone());

    match command {
        Commands::Serve(params) => serve(schema, control_state, params).await?,
        Commands::Sdl => {
            println!(
                "{}",
                schema.sdl_with_options(SDLExportOptions::new().prefer_single_line_descriptions())
            );
        }
    }

    Ok(())
}
