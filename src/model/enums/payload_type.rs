use async_graphql::Enum;
use clap::ValueEnum;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq, ValueEnum)]
pub enum PayloadType {
    Request,

    OnlyData,
    OnlyError,
    PartialDataAndError,

    NonGraphQl,
}

impl PayloadType {
    pub fn from_json(payload: &serde_json::Value) -> PayloadType {
        if let serde_json::Value::Object(map) = payload {
            let contains_query = map.contains_key("query");
            let contains_data = map.contains_key("data");
            let contains_errors = map.contains_key("errors");

            if contains_query {
                PayloadType::Request
            } else if contains_data && contains_errors {
                PayloadType::PartialDataAndError
            } else if contains_data {
                PayloadType::OnlyData
            } else if contains_errors {
                PayloadType::OnlyError
            } else {
                PayloadType::NonGraphQl
            }
        } else {
            PayloadType::NonGraphQl
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_payload() {}

    #[test]
    fn test_http_graphql_payload() {
        let query = serde_json::json!({ "query": "test" });
        let data = serde_json::json!({ "data": "test" });
        let partial_data = serde_json::json!({ "data": "test", "errors": "test" });
        let error = serde_json::json!({ "errors": "test" });
        let non_graphql = serde_json::json!("foobar");

        assert_eq!(PayloadType::from_json(&query), PayloadType::Request);
        assert_eq!(PayloadType::from_json(&data), PayloadType::OnlyData);
        assert_eq!(
            PayloadType::from_json(&partial_data),
            PayloadType::PartialDataAndError
        );
        assert_eq!(PayloadType::from_json(&error), PayloadType::OnlyError);
        assert_eq!(
            PayloadType::from_json(&non_graphql),
            PayloadType::NonGraphQl
        );
    }
}
