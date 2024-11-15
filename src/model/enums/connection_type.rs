use async_graphql::Enum;
use clap::ValueEnum;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum ConnectionType {
    Http,
    Ws,
}
