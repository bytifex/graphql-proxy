use async_graphql::Enum;
use clap::ValueEnum;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, ValueEnum, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageDirection {
    Request,
    Response,
}
