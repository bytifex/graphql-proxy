use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use http::HeaderValue;

#[derive(Debug, Clone)]
pub struct HeaderValueScalar(pub HeaderValue);

impl HeaderValueScalar {
    pub fn as_header_value(&self) -> &HeaderValue {
        &self.0
    }
}

impl From<HeaderValue> for HeaderValueScalar {
    fn from(value: HeaderValue) -> Self {
        Self(value)
    }
}

#[Scalar(name = "HeaderValue")]
impl ScalarType for HeaderValueScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(val) => Ok(HeaderValue::try_from(val)
                .map_err(|_e| InputValueError::expected_type(value))?
                .into()),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        if let Ok(val) = self.0.to_str() {
            Value::String(val.to_string())
        } else {
            Value::Binary(self.0.as_bytes().to_vec().into())
        }
    }
}
