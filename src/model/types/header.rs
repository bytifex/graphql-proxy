use async_graphql::Object;

use crate::model::scalars::{
    header_name_scalar::HeaderNameScalar, header_value_scalar::HeaderValueScalar,
};

#[derive(Debug, Clone)]
pub struct Header {
    pub name: HeaderNameScalar,
    pub value: HeaderValueScalar,
}

#[Object]
impl Header {
    async fn name(&self) -> &HeaderNameScalar {
        &self.name
    }

    async fn value(&self) -> &HeaderValueScalar {
        &self.value
    }
}
