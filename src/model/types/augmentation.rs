use async_graphql::Object;

use crate::error::UnimplementedError;

pub struct Augmentation;

#[Object]
impl Augmentation {
    pub async fn name(&self) -> Result<String, UnimplementedError> {
        Err(UnimplementedError("Augmentation::name".into()))
    }

    pub async fn description(&self) -> Result<String, UnimplementedError> {
        Err(UnimplementedError("Augmentation::description".into()))
    }
}
