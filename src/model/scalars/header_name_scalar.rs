use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use http::HeaderName;

#[derive(Debug, Clone)]
pub struct HeaderNameScalar(pub HeaderName);

impl HeaderNameScalar {
    pub fn as_header_name(&self) -> &HeaderName {
        &self.0
    }
}

impl From<HeaderName> for HeaderNameScalar {
    fn from(value: HeaderName) -> Self {
        Self(value)
    }
}

#[Scalar(name = "HeaderName")]
impl ScalarType for HeaderNameScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(val) => Ok(HeaderName::try_from(val)
                .map_err(|_e| InputValueError::expected_type(value))?
                .into()),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}
