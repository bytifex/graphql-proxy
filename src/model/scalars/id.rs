use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[derive(Debug, Clone)]
pub struct Id(pub String);

impl Id {
    pub fn as_string_ref(&self) -> &String {
        &self.0
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[Scalar(name = "ID")]
impl ScalarType for Id {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(val) => Ok(val.into()),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}
