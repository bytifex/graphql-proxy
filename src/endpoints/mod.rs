mod graphql_proxy;
mod graphql_ws_proxy;
pub mod router;

use async_graphql::http::GraphiQLSource;
use axum::response::Html;

pub async fn options_graphql() {}

pub async fn get_index() -> Html<String> {
    Html(
        r#"
            <html>
                <body>
                    <a href="/admin-graphiql">Admin GraphiQL</a>
                    <a href="/graphiql">Proxied GraphiQL</a>
                </body>
            </html>
        "#
        .into(),
    )
}

pub async fn graphiql(
    graphql_endpoint: impl AsRef<str>,
    graphql_ws_endpoint: impl AsRef<str>,
) -> Html<String> {
    Html(
        GraphiQLSource::build()
            .endpoint(graphql_endpoint.as_ref())
            .subscription_endpoint(graphql_ws_endpoint.as_ref())
            .finish(),
    )
}
